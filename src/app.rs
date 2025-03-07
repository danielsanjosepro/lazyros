use std::{io, sync::mpsc};

use crate::event::Event;

use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Widget},
    DefaultTerminal, Frame,
};

enum Movement {
    Up,
    Down,
    Left,
    Right,
}

pub struct App {
    exit: bool,
    topics: Vec<String>,
    nodes: Vec<String>,
    active_window: String,
    details: String,
}

impl App {
    pub fn new() -> App {
        App {
            exit: false,
            topics: vec!["/topic1".to_string(), "/topic2".to_string()],
            nodes: vec!["/node1".to_string(), "/node2".to_string()],
            active_window: "nodes".to_string(),
            details: "".to_string(),
        }
    }

    /// Main task to be run continuously
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

        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('n') => self.active_window = "nodes".to_string(),
            KeyCode::Char('t') => self.active_window = "topics".to_string(),

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
            Movement::Up => {
                if self.active_window == "topics" {
                    self.active_window = "nodes".to_string();
                }
            }
            Movement::Down => {
                if self.active_window == "nodes" {
                    self.active_window = "topics".to_string();
                }
            }
            Movement::Right => self.active_window = "details".to_string(),
            Movement::Left => self.active_window = "nodes".to_string(),
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

        let nodes_border = Block::bordered()
            .title(Line::from(" Nodes "))
            .border_set(border::ROUNDED)
            .style(if self.active_window == "nodes" {
                Color::Blue
            } else {
                Color::White
            });

        List::new(self.nodes.iter().map(|node| {
            ListItem::new(Line::from(node.clone()).style(Style::default().fg(Color::White)))
        }))
        .block(nodes_border)
        .render(nodes_area, buf);

        let topics_border = Block::bordered()
            .title(Line::from(" Topics "))
            .border_set(border::ROUNDED)
            .style(if self.active_window == "topics" {
                Color::Blue
            } else {
                Color::White
            });

        List::new(self.topics.iter().map(|topic| {
            ListItem::new(Line::from(topic.clone()).style(Style::default().fg(Color::White)))
        }))
        .block(topics_border)
        .render(topics_area, buf);

        let details_block = Block::bordered()
            .title(Line::from(" Details - TODO "))
            .border_set(border::ROUNDED)
            .style(if self.active_window == "details" {
                Color::Blue
            } else {
                Color::White
            });

        Paragraph::new(self.details.clone())
            .block(details_block)
            .render(details_area, buf);

        Line::from(vec![" Quit ".into(), "<q>".blue().bold()])
            .centered()
            .bold()
            .render(instructions_area, buf);
    }
}
