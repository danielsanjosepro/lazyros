use futures::{executor::LocalPool, future, stream::StreamExt, task::LocalSpawnExt};
use r2r::QosProfile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "lazyros", "")?;
    let subscriber =
        node.subscribe::<r2r::std_msgs::msg::String>("/topic", QosProfile::default())?;

    // Set up a simple task executor.
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    // Run the subscriber in one task, printing the messages
    spawner.spawn_local(async move {
        subscriber
            .for_each(|msg| {
                println!("got new msg: {}", msg.data);
                future::ready(())
            })
            .await
    })?;

    // Main loop spins ros.
    loop {
        node.spin_once(std::time::Duration::from_millis(100));
        pool.run_until_stalled();
    }
}

