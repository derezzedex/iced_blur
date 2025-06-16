use iced::widget::{button, column, container, float, stack, text};
use iced::{Center, Element};
use iced_blur::blur;

pub fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let background = column![
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement)
        ]
        .padding(20)
        .align_x(Center);

        let overlay = container(blur(1, text("h").size(20))).padding(20);

        stack![
            background,
            float(container(text("mid").size(20)).padding(10)),
            overlay,
        ]
        .into()
    }
}
