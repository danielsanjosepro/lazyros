pub enum Event {
    Input(crossterm::event::KeyEvent),
    Resize(u16, u16),
    ROSEvent { event: ROSEvent },
}

pub enum ROSEvent {
    SubscriptionMessage(String),
    NewNode(String),
    NewTopic(String, String),
}
