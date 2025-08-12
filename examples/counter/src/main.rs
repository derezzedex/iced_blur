use iced::widget::{
    button, column, container, float, horizontal_space, row, slider, stack, text, toggler,
};
use iced::{Center, Element};
use iced_blur::blur;

pub fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}

#[derive(Default)]
struct Counter {
    show: bool,
    radius: u32,
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ToggleBlur(bool),
    BlurRadiusChanged(u32),
    Increment,
    Decrement,
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleBlur(show) => {
                self.show = show;
            }
            Message::BlurRadiusChanged(radius) => {
                self.radius = radius;
            }
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            column![
                text!("Radius: {}", self.radius),
                slider(0..=20, self.radius, Message::BlurRadiusChanged),
            ]
            .spacing(4),
            column![
                text("Blur"),
                toggler(self.show).on_toggle(Message::ToggleBlur)
            ]
            .spacing(4),
            horizontal_space(),
        ]
        .padding(8)
        .spacing(8);

        let background = column![
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement)
        ]
        .padding(20)
        .align_x(Center);

        column![
            controls,
            stack![
                background,
                float(container(text("mid").size(20)).padding(10)),
                self.show.then(|| blur(self.radius)),
                container(text("h").size(20))
                    .width(50)
                    .height(50)
                    .padding(10),
            ]
        ]
        .into()
    }
}
