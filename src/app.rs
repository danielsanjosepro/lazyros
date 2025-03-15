use std::{io, sync::mpsc};

use crate::event::{self, Event};

use ratatui::{
    layout::{Constraint, Layout, Margin, Offset},
    prelude::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Padding, Paragraph, Row, ScrollbarState, TableState, Widget},
    DefaultTerminal, Frame,
};

const ITEM_HEIGHT: usize = 1;

enum Movement {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Default)]
pub struct App {
    app_state: AppState,

    details: String,
    instructions: Vec<Instruction>,
    pane_manager: PaneManager,
}

#[derive(Debug, Default, Eq, PartialEq)]
enum AppState {
    #[default]
    Navigation,
    ShowingInstructions,
    ActivePane,
    Exit,
}

#[derive(Debug, Default)]
struct PaneManager {
    node_pane: NodePane,
    topics_pane: TopicPane,
    details_pane: String,
    focused_pane: PaneType,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum PaneType {
    #[default]
    NodePane,
    TopicsPane,
    DetailsPane,
}

impl PaneManager {
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key_event.code {
            KeyCode::Left | KeyCode::Char('h') => self.previous_pane(),
            KeyCode::Right | KeyCode::Char('l') => self.next_pane(),
            KeyCode::Char('n') => self.focused_pane = PaneType::NodePane,
            KeyCode::Char('t') => self.focused_pane = PaneType::TopicsPane,
            KeyCode::Char('d') => self.focused_pane = PaneType::DetailsPane,
            _ => {}
        }

        Ok(())
    }

    fn previous_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            PaneType::NodePane => PaneType::DetailsPane,
            PaneType::TopicsPane => PaneType::NodePane,
            PaneType::DetailsPane => PaneType::TopicsPane,
        }
    }

    fn next_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            PaneType::NodePane => PaneType::TopicsPane,
            PaneType::TopicsPane => PaneType::DetailsPane,
            PaneType::DetailsPane => PaneType::NodePane,
        }
    }
}

struct Instruction {
    key_code: crossterm::event::KeyCode,
    description: String,
}

impl Instruction {
    pub fn new(key_code: char, text: &str) -> Instruction {
        Instruction {
            key_code: crossterm::event::KeyCode::Char(key_code),
            description: text.to_string(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct TopicPane {
    state: TableState,
    scroll_state: ScrollbarState,
    topics: Vec<TopicData>,
}

#[derive(Debug, Default, Eq, PartialEq)]
struct TopicData {
    name: String,
    msg_type: String,
    num_subscribers: u32,
}

impl TopicData {
    fn as_vec_string(&self) -> Vec<String> {
        return vec![
            self.name.clone(),
            self.msg_type.clone(),
            self.num_subscribers.to_string(),
        ];
    }
}

impl TopicPane {
    pub fn get_rows(&self) -> Vec<Row> {
        // Get first the data as a string
        let topics_data_string: Vec<Vec<String>> = self.iter().map(|t| t.as_vec_string()).collect();

        topics_data_string
            .into_iter()
            .map(|topic_data_string| {
                Row::new(topic_data_string).style(Style::default().fg(Color::White))
            })
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TopicData> {
        self.topics.iter()
    }

    pub fn add_topic(&mut self, topic: TopicData) {
        self.topics.push(topic);
        self.scroll_state = ScrollbarState::new(self.topics.len());
    }

    pub fn remove_topic(&mut self, topic: TopicData) {
        self.topics.retain(|t| t != &topic);
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.topics.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.topics.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_column(&mut self) {
        self.state.select_next_column();
    }

    pub fn previous_column(&mut self) {
        self.state.select_previous_column();
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => self.handle_arrow(Movement::Up)?,
            KeyCode::Down | KeyCode::Char('j') => self.handle_arrow(Movement::Down)?,
            KeyCode::Left | KeyCode::Char('h') => self.handle_arrow(Movement::Left)?,
            KeyCode::Right | KeyCode::Char('l') => self.handle_arrow(Movement::Right)?,
            _ => {}
        }

        Ok(())
    }

    fn handle_arrow(&mut self, movement: Movement) -> io::Result<()> {
        match movement {
            Movement::Up => self.previous_row(),
            Movement::Down => self.next_row(),
            Movement::Left => self.previous_column(),
            Movement::Right => self.next_column(),
        }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct NodePane {
    state: TableState,
    scroll_state: ScrollbarState,
    nodes: Vec<NodeData>,
}

#[derive(Debug, Default, Eq, PartialEq)]
struct NodeData {
    name: String,
}

impl NodePane {
    pub fn iter(&self) -> impl Iterator<Item = &NodeData> {
        self.nodes.iter()
    }

    pub fn add_node(&mut self, node: NodeData) {
        self.nodes.push(node);
        self.scroll_state = ScrollbarState::new(self.nodes.len());
    }

    pub fn remove_node(&mut self, node: NodeData) {
        self.nodes.retain(|n| n != &node);
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.nodes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.nodes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_column(&mut self) {
        self.state.select_next_column();
    }

    pub fn previous_column(&mut self) {
        self.state.select_previous_column();
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => self.handle_arrow(Movement::Up)?,
            KeyCode::Down | KeyCode::Char('j') => self.handle_arrow(Movement::Down)?,
            KeyCode::Left | KeyCode::Char('h') => self.handle_arrow(Movement::Left)?,
            KeyCode::Right | KeyCode::Char('l') => self.handle_arrow(Movement::Right)?,
            _ => {}
        }

        Ok(())
    }

    fn handle_arrow(&mut self, movement: Movement) -> io::Result<()> {
        match movement {
            Movement::Up => self.previous_row(),
            Movement::Down => self.next_row(),
            Movement::Left => self.previous_column(),
            Movement::Right => self.next_column(),
        }
        Ok(())
    }
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            app_state: AppState::default(),
            details: "".to_string(),
            instructions: vec![
                Instruction::new('q', "Return to navigation"),
                Instruction::new('i', "Toggle instructions"),
                Instruction::new('j', "Down"),
                Instruction::new('k', "Up"),
                Instruction::new('h', "Left"),
                Instruction::new('l', "Right"),
                Instruction {
                    key_code: crossterm::event::KeyCode::Enter,
                    description: "Focus window".to_string(),
                },
                Instruction {
                    key_code: crossterm::event::KeyCode::Esc,
                    description: "Escape focused window".to_string(),
                },
            ],
            pane_manager: PaneManager::default(),
        };
        // Add more test nodes
        for i in 1..15 {
            app.pane_manager.node_pane.add_node(NodeData {
                name: format!("/node{}", i),
            });
        }
        app
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        while self.app_state != AppState::Exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Resize(_, _) => terminal.clear()?,
                // TODO: handle resize, for now only
                // render the terminal again
                Event::ROSEvent { event: ros_event } => self.handle_ros_events(ros_event)?,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    /// Render `self`, as we implemented the Widget trait for &App
    fn draw(&self, frame: &mut Frame) {
        // Split main layout into content and instructions
        let main_layout = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)]);
        let [main_area, instructions_area] = main_layout.areas(frame.area());

        self.render_main_content(main_area, frame);
        self.render_instructions_bar(instructions_area, frame);

        // Render instructions popup if needed
        if self.app_state == AppState::ShowingInstructions {
            self.render_instructions_popup(frame.area(), frame);
        }
    }

    fn handle_ros_events(&mut self, ros_event: event::ROSEvent) -> io::Result<()> {
        match ros_event {
            event::ROSEvent::SubscriptionMessage(msg) => {
                self.details += &msg;
                self.details += "\n";
            }
            event::ROSEvent::NewNode(name) => {
                self.pane_manager.node_pane.add_node(NodeData { name });
            }
            event::ROSEvent::NewTopic(name, msg_type) => {
                self.pane_manager.topics_pane.add_topic(TopicData {
                    name,
                    num_subscribers: 0,
                    msg_type,
                });
            }
        }
        return Ok(());
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }

        match (&self.app_state, key_event.code) {
            (AppState::Navigation | AppState::ActivePane, KeyCode::Char('i')) => {
                self.app_state = AppState::ShowingInstructions
            }

            (_, KeyCode::Char('q')) => self.app_state = AppState::Exit,

            (AppState::ShowingInstructions, KeyCode::Esc) => self.app_state = AppState::Navigation,
            (AppState::ShowingInstructions, KeyCode::Char('i')) => {
                self.app_state = AppState::Navigation
            }

            (AppState::Navigation, KeyCode::Enter) => self.app_state = AppState::ActivePane,
            (AppState::ActivePane, KeyCode::Esc) => {
                self.app_state = AppState::Navigation;
            }

            (AppState::ActivePane, _) => match self.pane_manager.focused_pane {
                PaneType::NodePane => self.pane_manager.node_pane.handle_key_event(key_event)?,
                PaneType::TopicsPane => {
                    self.pane_manager.topics_pane.handle_key_event(key_event)?
                }
                _ => {}
            },
            (AppState::Navigation, _) => self.pane_manager.handle_key_event(key_event)?,
            (AppState::ShowingInstructions, _) => {}
            (AppState::Exit, _) => {}
        }

        Ok(())
    }
}

impl App {
    fn render_main_content(&self, area: Rect, frame: &mut Frame) {
        let left_right_layout =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
        let [options_area, details_area] = left_right_layout.areas(area);

        self.render_options_panes(options_area, frame);
        self.render_details_pane(details_area, frame);
    }

    fn render_options_panes(&self, area: Rect, frame: &mut Frame) {
        let options_layout =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [nodes_area, topics_area] = options_layout.areas(area);

        self.render_nodes_pane(nodes_area, frame);
        self.render_topics_pane(topics_area, frame);
    }

    fn render_nodes_pane(&self, area: Rect, frame: &mut Frame) {
        use ratatui::widgets::{Row, Scrollbar, ScrollbarOrientation, Table};

        // Render the border first
        let is_focused = self.pane_manager.focused_pane == PaneType::NodePane;
        let is_active = self.app_state == AppState::ActivePane;
        let block = create_stylized_block(" Nodes ", is_focused, is_active);
        frame.render_widget(block, area);

        // We split the area in scrollable area and scrollbar
        let inner_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        let left_right_layout =
            Layout::horizontal([Constraint::Percentage(100), Constraint::Min(1)]);
        let [scrollable_area, scrollbar_area] = left_right_layout.areas(inner_area);

        let header = Row::new(vec!["Node Name"]).style(Style::default().fg(Color::Yellow));

        let rows: Vec<Row> = self
            .pane_manager
            .node_pane
            .iter()
            .map(|node| Row::new(vec![node.name.as_str()]).style(Style::default().fg(Color::White)))
            .collect();

        //let cols = vec!["Node Name"];

        let table = Table::default()
            .header(header)
            .row_highlight_style(Style::default().fg(Color::Green).bold())
            .rows(rows);

        frame.render_stateful_widget(
            table,
            scrollable_area,
            &mut self.pane_manager.node_pane.state.clone(),
        );

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        frame.render_stateful_widget(
            scrollbar,
            scrollbar_area,
            &mut self.pane_manager.node_pane.scroll_state.clone(),
        );
    }

    fn render_topics_pane(&self, area: Rect, frame: &mut Frame) {
        use ratatui::widgets::{Row, Scrollbar, ScrollbarOrientation, Table};

        let is_focused = self.pane_manager.focused_pane == PaneType::TopicsPane;
        let is_active = self.app_state == AppState::ActivePane;
        let block = create_stylized_block(" Topics ", is_focused, is_active);
        frame.render_widget(block, area);

        // We split the area in scrollable area and scrollbar
        let inner_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        let left_right_layout =
            Layout::horizontal([Constraint::Percentage(100), Constraint::Min(1)]);
        let [scrollable_area, scrollbar_area] = left_right_layout.areas(inner_area);

        let header = Row::new(vec!["Topic Name", "Message Type", "Publisher Count"])
            .style(Style::default().fg(Color::Yellow));

        let rows = self.pane_manager.topics_pane.get_rows();

        let table = Table::default()
            .header(header)
            .row_highlight_style(Style::default().fg(Color::Green).bold())
            .rows(rows);

        // Render table with state
        frame.render_stateful_widget(
            table,
            scrollable_area,
            &mut self.pane_manager.topics_pane.state.clone(),
        );

        // Render scrollbar
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        frame.render_stateful_widget(
            scrollbar,
            scrollbar_area,
            &mut self.pane_manager.topics_pane.scroll_state.clone(),
        );
    }

    fn render_details_pane(&self, area: Rect, frame: &mut Frame) {
        let details = Paragraph::new(self.details.clone()).block(create_stylized_block(
            " Details area ",
            self.pane_manager.focused_pane == PaneType::DetailsPane,
            self.app_state == AppState::ActivePane,
        ));

        details.render(area, frame.buffer_mut());
    }

    fn render_instructions_bar(&self, area: Rect, frame: &mut Frame) {
        let instructions_line = Line::from(vec![
            " Quit ".into(),
            "<q>".blue().bold(),
            " Instructions ".into(),
            "<i>".blue().bold(),
        ])
        .centered()
        .bold();

        instructions_line.render(area, frame.buffer_mut());
    }

    fn render_instructions_popup(&self, area: Rect, frame: &mut Frame) {
        let instructions: Vec<Line> = self
            .instructions
            .iter()
            .map(|instruction| {
                Line::from(vec![
                    format!("{:?}", instruction.key_code).into(),
                    " - ".into(),
                    instruction.description.clone().into(),
                ])
            })
            .collect();

        let instructions_paragraph =
            Paragraph::new(instructions).block(Block::bordered().title("Instructions"));

        let popup_area = popup_area(area, 60, 20);
        ratatui::widgets::Clear.render(popup_area, frame.buffer_mut());
        instructions_paragraph.render(popup_area, frame.buffer_mut());
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical =
        Layout::vertical([Constraint::Percentage(percent_y)]).flex(ratatui::layout::Flex::Center);
    let horizontal =
        Layout::horizontal([Constraint::Percentage(percent_x)]).flex(ratatui::layout::Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn create_stylized_block(
    title: &str,
    is_focused: bool,
    is_active: bool,
) -> ratatui::widgets::Block {
    let border_style = match { is_focused && is_active } {
        true => BorderType::Thick,
        false => BorderType::Rounded,
    };
    let color = match (is_focused, is_active) {
        (true, true) => Color::Green,
        (true, false) => Color::Blue,
        (false, _) => Color::White,
    };

    Block::bordered()
        .title(Line::from(title))
        .style(color)
        .border_type(border_style)
}
