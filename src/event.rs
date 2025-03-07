pub enum Event {
    Input(crossterm::event::KeyEvent), // crossterm key input event
    Resize(u16, u16),                  // crossterm resize event
    ROSEvent(String),                  // progress update from the computation thread
}
