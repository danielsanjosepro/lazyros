use futures::{future, stream::StreamExt};
use r2r::QosProfile;
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    thread,
};
use tokio::task;

mod app;
mod event;

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

#[tokio::main]
async fn run_ros_thread(tx: mpsc::Sender<event::Event>) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let node = Arc::new(Mutex::new(r2r::Node::create(ctx, "lazyros", "")?));

    let sub_node = node.clone();
    let sub_tx = tx.clone();
    let timer_node = node.clone();
    let timer_tx = tx.clone();

    tx.send(event::Event::ROSEvent {
        event: event::ROSEvent::SubscriptionMessage("Subscribing to /topic".to_string()),
    })?;

    task::spawn(async move { subscribe(sub_node, sub_tx).await.unwrap() });
    task::spawn(async move { send_topics(timer_node, timer_tx).await.unwrap() });

    let handle = tokio::task::spawn_blocking(move || loop {
        node.lock()
            .unwrap()
            .spin_once(std::time::Duration::from_millis(10));
        std::thread::sleep(std::time::Duration::from_millis(100));
    });

    handle.await?;

    Ok(())
}

async fn subscribe(
    arc_node: Arc<Mutex<r2r::Node>>,
    tx: mpsc::Sender<event::Event>,
) -> Result<(), r2r::Error> {
    let subscriber = arc_node
        .lock()
        .unwrap()
        .subscribe::<r2r::std_msgs::msg::String>("/topic", QosProfile::default())?;

    subscriber
        .for_each(|msg| {
            let _ = tx.send(event::Event::ROSEvent {
                event: event::ROSEvent::SubscriptionMessage(msg.data),
            });
            future::ready(())
        })
        .await;
    Ok(())
}

async fn send_topics(
    arc_node: Arc<Mutex<r2r::Node>>,
    tx: mpsc::Sender<event::Event>,
) -> Result<(), r2r::Error> {
    let mut timer = arc_node
        .lock()
        .unwrap()
        .create_wall_timer(std::time::Duration::from_secs(1))
        .unwrap();

    let mut topics: Vec<String> = vec![];

    loop {
        if let Err(_e) = timer.tick().await {
            // Handle error
        };

        // Get the latest node value
        let detected_topic_names = arc_node
            .lock()
            .unwrap()
            .get_topic_names_and_types()
            .unwrap();

        // find new topics
        let new_topics: Vec<String> = detected_topic_names
            .iter()
            .filter(|(topic, _)| !topics.contains(*topic))
            .map(|(topic, _)| topic.clone())
            .collect();

        for topic in &new_topics {
            topics.push(topic.clone());
            let new_msg_data = detected_topic_names
                .get(topic.as_str())
                .unwrap()
                .get(0)
                .unwrap();

            if let Err(_e) = tx.send(event::Event::ROSEvent {
                event: event::ROSEvent::NewTopic(topic.clone(), new_msg_data.to_string()),
            }) {
                // Handle error
            }
        }
    }
}
