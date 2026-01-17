use iced::widget::{button, column, text};
use iced::Element;

fn main() -> iced::Result {
    iced::run("Test Window", update, view)
}

#[derive(Default)]
struct State {
    count: i32,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::Increment => state.count += 1,
    }
}

fn view(state: &State) -> Element<Message> {
    column![
        text(format!("Count: {}", state.count)).size(50),
        button("Increment").on_press(Message::Increment),
    ]
    .padding(20)
    .into()
}
