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
    passes: u32,
    offset: f32,
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ToggleBlur(bool),
    BlurOffsetChanged(f32),
    BlurPassesChanged(u32),
    Increment,
    Decrement,
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleBlur(show) => {
                self.show = show;
            }
            Message::BlurPassesChanged(passes) => {
                self.passes = passes;
            }
            Message::BlurOffsetChanged(offset) => {
                self.offset = offset;
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
                column![
                    text!("Passes: {}", self.passes),
                    slider(0..=20, self.passes, Message::BlurPassesChanged),
                ]
                .spacing(4),
                column![
                    text!("Offset: {}", self.offset),
                    slider(0f32..=50.0, self.offset, Message::BlurOffsetChanged).step(0.1),
                ]
                .spacing(4),
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
                self.show.then(|| blur(self.passes, self.offset)),
                container(text("h").size(20))
                    .width(50)
                    .height(50)
                    .padding(10),
            ]
        ]
        .into()
    }
}
