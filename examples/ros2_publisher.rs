fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "lazyros", "")?;
    let publisher =
        node.create_publisher::<r2r::std_msgs::msg::String>("/topic", r2r::QosProfile::default())?;

    println!("Publishing to /topic in 5 seconds.");

    std::thread::sleep(std::time::Duration::from_millis(5000));

    let string_msg = r2r::std_msgs::msg::String {
        data: "Hello world!".to_string(),
        ..Default::default()
    };

    publisher.publish(&string_msg).unwrap();

    node.destroy_publisher(publisher);

    println!("Published a message!");

    Ok(())
}
