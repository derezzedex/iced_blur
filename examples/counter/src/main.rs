use iced::widget::{
    button, column, container, float, row, slider, stack, text, toggler, vertical_rule,
};
use iced::{Alignment, Center, Element, Length};
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

fn gaussian_blur(radius: f32) -> (u32, f32) {
    let passes = (((4.0 / 3.0) * radius.log2()).round() as u32).max(1);
    let offset = 0.4538f32.powi(passes as i32) * radius;

    (passes, offset)
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

                let (passes, offset) = gaussian_blur(radius);
                self.passes = passes;
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
                text("Blur"),
                toggler(self.show).on_toggle(Message::ToggleBlur)
            ]
            .spacing(4),
            column![
                text!("Radius: {}", self.radius),
                row![
                    button("-")
                        .padding([0, 5])
                        .on_press(Message::BlurRadiusChanged(self.radius - 0.01)),
                    slider(0f32..=25.0, self.radius, Message::BlurRadiusChanged).step(0.01),
                    button("+")
                        .padding([0, 5])
                        .on_press(Message::BlurRadiusChanged(self.radius + 0.01)),
                ]
                .align_y(Alignment::Center)
                .spacing(4)
            ]
            .spacing(4),
            vertical_rule(2),
            column![
                text!("Passes: {}", self.passes),
                row![
                    button("-")
                        .padding([0, 5])
                        .on_press(Message::BlurPassesChanged(self.passes.saturating_sub(1))),
                    slider(0..=20, self.passes, Message::BlurPassesChanged),
                    button("+")
                        .padding([0, 5])
                        .on_press(Message::BlurPassesChanged(self.passes + 1)),
                ]
                .align_y(Alignment::Center)
                .spacing(4)
            ]
            .spacing(4),
            column![
                text!("Offset: {}", self.offset),
                row![
                    button("-")
                        .padding([0, 5])
                        .on_press(Message::BlurOffsetChanged(self.offset - 0.01)),
                    slider(0f32..=2.0, self.offset, Message::BlurOffsetChanged).step(0.01),
                    button("+")
                        .padding([0, 5])
                        .on_press(Message::BlurOffsetChanged(self.offset + 0.01)),
                ]
                .align_y(Alignment::Center)
                .spacing(4)
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
