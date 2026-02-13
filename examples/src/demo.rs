use iced::Task;
use iced_layout::{hot_layout, layout};

fn main() -> iced::Result {
    iced::application(Application::boot, Application::update, Application::render)
        .subscription(Application::subscription)
        .run()
}

struct Application;

#[derive(Debug, Clone)]
enum Message {
    LayoutChanged,
}

impl Application {
    pub fn boot() -> (Self, Task<Message>) {
        (Self, Task::none())
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced_layout::hot_reload_subscription().map(|_| Message::LayoutChanged)
    }

    pub fn render(&self) -> iced::Element<'_, Message> {
        hot_layout("page/test-layout.xml")
    }
}
