use futures::{executor::LocalPool, future, stream::StreamExt, task::LocalSpawnExt};
use std::{io, sync::mpsc, thread};

mod app;
mod event;
use r2r::QosProfile;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = mpsc::channel::<event::Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_background_progress_events = event_tx.clone();
    thread::spawn(move || {
        run_ros_thread(tx_to_background_progress_events).unwrap();
    });

    let mut app = app::App::new();

    let app_result = app.run(&mut terminal, event_rx);

    ratatui::restore();
    app_result
}

/// Block, waiting for input events from the user.
fn handle_input_events(tx: mpsc::Sender<event::Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(event::Event::Input(key_event)).unwrap()
            }
            crossterm::event::Event::Resize(cols, rows) => {
                tx.send(event::Event::Resize(cols, rows)).unwrap()
            }
            _ => {}
        }
    }
}

///
fn run_ros_thread(tx: mpsc::Sender<event::Event>) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "lazyros", "")?;
    let subscriber =
        node.subscribe::<r2r::std_msgs::msg::String>("/topic", QosProfile::default())?;

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    tx.send(event::Event::ROSEvent("Subscribing to /topic".to_string()))
        .unwrap();
    spawner.spawn_local(async move {
        subscriber
            .for_each(|msg| {
                tx.send(event::Event::ROSEvent(msg.data)).unwrap();
                future::ready(())
            })
            .await
    })?;

    loop {
        node.spin_once(std::time::Duration::from_millis(100));
        pool.run_until_stalled();
    }
}
