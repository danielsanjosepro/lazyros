use std::{io, sync::mpsc};

use crate::event::Event;

use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, List, ListItem, Paragraph, Widget},
    DefaultTerminal, Frame,
};

enum Movement {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Default)]
pub struct App {
    exit: bool,
    topics: Vec<String>,
    nodes: Vec<String>,
    focused_window: String,
    window_active: bool,
    details: String,
    instructions_visible: bool,
    instructions: Vec<Instruction>,
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

impl App {
    pub fn new() -> App {
        App {
            exit: false,
            topics: vec!["/topic1".to_string(), "/topic2".to_string()],
            nodes: vec!["/node1".to_string(), "/node2".to_string()],
            focused_window: "nodes".to_string(),
            window_active: false,
            details: "".to_string(),
            instructions_visible: false,
            instructions: vec![
                Instruction::new('q', "Quit"),
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
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Resize(_, _) => terminal.clear()?,
                // TODO: handle resize, for now only
                // render the terminal again
                Event::ROSEvent(ros_event) => self.handle_ros_events(ros_event)?,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    /// Render `self`, as we implemented the Widget trait for &App
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_ros_events(&mut self, ros_event: String) -> io::Result<()> {
        self.details += "\n";
        self.details += &ros_event;
        return Ok(());
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }

        match (key_event.code, self.window_active) {
            (KeyCode::Char('i'), _) => self.instructions_visible = !self.instructions_visible,
            (KeyCode::Char('q'), _) => self.exit = true,
            //(KeyCode::Char('n') | _) => self.focused_window = "nodes".to_string(),
            //(KeyCode::Char('t') | _) => self.focused_window = "topics".to_string(),
            (KeyCode::Up | KeyCode::Char('k'), false) => self.handle_arrow(Movement::Up)?,
            (KeyCode::Down | KeyCode::Char('j'), false) => self.handle_arrow(Movement::Down)?,
            (KeyCode::Left | KeyCode::Char('h'), false) => self.handle_arrow(Movement::Left)?,
            (KeyCode::Right | KeyCode::Char('l'), false) => self.handle_arrow(Movement::Right)?,

            (KeyCode::Enter, _) => self.window_active = !self.window_active,

            _ => {}
        }

        Ok(())
    }

    fn handle_arrow(&mut self, movement: Movement) -> io::Result<()> {
        match movement {
            Movement::Up => {
                if self.focused_window == "topics" {
                    self.focused_window = "nodes".to_string();
                }
            }
            Movement::Down => {
                if self.focused_window == "nodes" {
                    self.focused_window = "topics".to_string();
                }
            }
            Movement::Right => self.focused_window = "details".to_string(),
            Movement::Left => self.focused_window = "nodes".to_string(),
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)]);
        let [main_area, instructions_area] = main_layout.areas(area);
        let left_right_layout =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
        let [options_area, details_area] = left_right_layout.areas(main_area);
        let options_layout =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [nodes_area, topics_area] = options_layout.areas(options_area);

        List::new(self.nodes.iter().map(|node| {
            ListItem::new(Line::from(node.clone()).style(Style::default().fg(Color::White)))
        }))
        .block(create_stylized_block(
            " Nodes ",
            self.focused_window == "nodes",
            self.window_active,
        ))
        .render(nodes_area, buf);

        List::new(self.topics.iter().map(|topic| {
            ListItem::new(Line::from(topic.clone()).style(Style::default().fg(Color::White)))
        }))
        .block(create_stylized_block(
            " Topics ",
            self.focused_window == "topics",
            self.window_active,
        ))
        .render(topics_area, buf);

        Paragraph::new(self.details.clone())
            .block(create_stylized_block(
                " Details area ",
                self.focused_window == "details",
                self.window_active,
            ))
            .render(details_area, buf);

        Line::from(vec![
            " Quit ".into(),
            "<q>".blue().bold(),
            " Instructions ".into(),
            "<i>".blue().bold(),
        ])
        .centered()
        .bold()
        .render(instructions_area, buf);

        if self.instructions_visible {
            let block = Block::bordered().title("Instructions");
            let instructions_paragraph: _ = Paragraph::new(
                self.instructions
                    .iter()
                    .map(|instruction| {
                        Line::from(vec![
                            format!("{:?}", instruction.key_code).into(),
                            " - ".into(),
                            instruction.description.clone().into(),
                        ])
                    })
                    .collect::<Vec<_>>(),
            )
            .block(block);

            let instructions_popup_area = popup_area(area, 60, 20);
            ratatui::widgets::Clear.render(instructions_popup_area, buf);
            instructions_paragraph.render(instructions_popup_area, buf);
        }
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
    // TODO: use enums to handle focused and active windows
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
