use iced::widget::{column, text};
use iced::{Element, Task};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("JMAP Chat")
        .run()
}

struct App;

impl App {
    fn new() -> (Self, Task<()>) {
        (App, Task::none())
    }

    fn update(&mut self, _message: ()) {}

    fn view(&self) -> Element<'_, ()> {
        column![text("JMAP Chat").size(24), text("Coming soon"),].into()
    }
}
