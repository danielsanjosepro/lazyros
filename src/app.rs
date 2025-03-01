use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{executor::LocalPool, task::LocalSpawn};
use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};

#[derive()]
pub struct App {
    running: bool,
    node: r2r::Node,
    publisher: r2r::Publisher<r2r::std_msgs::msg::String>,
    available_topics: Vec<String>,
    pool: LocalPool,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let ctx = r2r::Context::create().unwrap();
        let mut node = r2r::Node::create(ctx.clone(), "lazyros", "").unwrap();
        let publisher = node
            .create_publisher::<r2r::std_msgs::msg::String>("/topic", r2r::QosProfile::default())
            .unwrap();

        let pool = LocalPool::new();
        //let spawner = pool.spawner();

        //spawner.spawn_local_obj(async move {
        //    //available_topics = node.get_topic_names_and_types();
        //})?;

        Self {
            running: false,
            node,
            publisher,
            available_topics: vec![],
            pool,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events()?;
            // Check for new nodes
            self.node.spin_once(std::time::Duration::from_millis(100));
            self.pool.run_until_stalled();

            let _map = self.node.get_topic_names_and_types().unwrap();
            for (topic, _) in _map {
                if !self.available_topics.contains(&topic) {
                    self.available_topics.push(topic);
                }
            }
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from("lazyros").bold().blue().centered();
        let text = "Welcome to lazyros!\n\n\
            Press `P` to publish 'Hello world!' to the topic /topic.\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running. \n\
            Current available topics:\n"
            .to_owned()
            + &self.available_topics.join("\n");

        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('P')) => self.publish(),
            _ => {}
        }
    }

    fn publish(&mut self) {
        let string_msg = r2r::std_msgs::msg::String {
            data: "Hello world!".to_string(),
            ..Default::default()
        };
        self.publisher.publish(&string_msg).unwrap();
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.node.destroy_publisher(self.publisher.clone());
        self.running = false;
    }
}
