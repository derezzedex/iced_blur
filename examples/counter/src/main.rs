use iced::widget::{
    button, column, container, float, row, slider, stack, text, toggler, vertical_rule,
};
use iced::{Center, Element, Length};
use iced_blur::blur;

pub fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}

#[derive(Default)]
struct Counter {
    show: bool,
    radius: f32,
    passes: u32,
    offset: f32,
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ToggleBlur(bool),
    BlurRadiusChanged(f32),
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
            Message::BlurRadiusChanged(radius) => {
                self.radius = radius;
                self.passes = (((4.0 / 3.0) * radius.log2()).round() as u32).max(1);
                self.offset = 0.4538f32.powi(self.passes as i32) * self.passes as f32;
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
                text("Blur"),
                toggler(self.show).on_toggle(Message::ToggleBlur)
            ]
            .spacing(4),
            column![
                text!("Radius: {}", self.radius),
                slider(0f32..=50.0, self.radius, Message::BlurRadiusChanged).step(0.1),
            ]
            .spacing(4),
            vertical_rule(2),
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
        .height(Length::Shrink)
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
            container(controls).style(container::dark),
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
