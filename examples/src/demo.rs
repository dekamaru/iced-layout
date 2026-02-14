use iced::Task;
use iced_layout::layout;

fn main() -> iced::Result {
    iced::application(Application::boot, Application::update, Application::render)
        .run()
}

struct Application {
    test: f32,
}

#[derive(Debug, Clone)]
enum Message {
    OnButtonClick,
}

impl Application {
    pub fn boot() -> (Self, Task<Message>) {
        (Self { test: 0.0 }, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OnButtonClick => {
                self.test += 1.0;
            }
        }

        Task::none()
    }

    fn on_press_with(&self) -> Message {
        Message::OnButtonClick
    }

    fn on_press_maybe(&self) -> Option<Message> {
        if self.test % 2.0 == 0.0 {
            Some(Message::OnButtonClick)
        } else {
            None
        }
    }

    pub fn render(&self) -> iced::Element<'_, Message> {
        layout!("page/test-layout.xml")
    }
}
