use iced::Task;
use iced_layout::layout;

fn main() -> iced::Result {
    iced::application(Application::boot, Application::update, Application::render)
        .run()
}

struct Application {
    my_variable: String,
}

#[derive(Debug, Clone)]
enum Message {}

impl Application {
    pub fn boot() -> (Self, Task<Message>) {
        (Self { my_variable: "variable content".to_string() }, Task::none())
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn render(&self) -> iced::Element<'_, Message> {
        layout!("page/test-layout.xml")
    }
}
