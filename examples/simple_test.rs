//! Simple Iced test example
//!
//! Requires the `iced-gui` feature to be enabled.

#[cfg(not(feature = "iced-gui"))]
fn main() {
    eprintln!("This example requires the `iced-gui` feature. Run with:");
    eprintln!("  cargo run --example simple_test --features iced-gui");
}

#[cfg(feature = "iced-gui")]
use iced::widget::{button, column, text};
#[cfg(feature = "iced-gui")]
use iced::Element;

#[cfg(feature = "iced-gui")]
fn main() -> iced::Result {
    iced::run("Test Window", update, view)
}

#[cfg(feature = "iced-gui")]
#[derive(Default)]
struct State {
    count: i32,
}

#[cfg(feature = "iced-gui")]
#[derive(Debug, Clone)]
enum Message {
    Increment,
}

#[cfg(feature = "iced-gui")]
fn update(state: &mut State, message: Message) {
    match message {
        Message::Increment => state.count += 1,
    }
}

#[cfg(feature = "iced-gui")]
fn view(state: &State) -> Element<Message> {
    column![
        text(format!("Count: {}", state.count)).size(50),
        button("Increment").on_press(Message::Increment),
    ]
    .padding(20)
    .into()
}
