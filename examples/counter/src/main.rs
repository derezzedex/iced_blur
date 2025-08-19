use iced::keyboard::{Key, on_key_release};
use iced::widget::{
    button, column, container, image, row, slider, stack, text, toggler, vertical_rule,
};
use iced::{Alignment, Color, Element, Length, Subscription, Task, window};
use iced_blur::blur;

pub fn main() -> iced::Result {
    iced::application(Counter::default, Counter::update, Counter::view)
        .centered()
        .subscription(Counter::subscription)
        .window_size([340.0, 340.0])
        .run()
}

#[derive(Default)]
struct Counter {
    controls: bool,
    show: bool,
    radius: f32,
    passes: u32,
    offset: f32,
}

#[derive(Debug, Clone)]
enum Message {
    Screenshot,
    ScreenshotTaken(window::Screenshot),
    ToggleControls,
    ToggleBlur(bool),
    BlurRadiusChanged(f32),
    BlurOffsetChanged(f32),
    BlurPassesChanged(u32),
}

fn gaussian_blur(radius: f32) -> (u32, f32) {
    let passes = (((4.0 / 3.0) * radius.log2()).round() as u32).max(1);
    let offset = 0.4538f32.powi(passes as i32) * radius;

    (passes, offset)
}

impl Counter {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Screenshot => {
                return window::get_latest()
                    .and_then(window::screenshot)
                    .map(Message::ScreenshotTaken);
            }
            Message::ScreenshotTaken(screenshot) => {
                let _ = ::image::save_buffer(
                    "assets/blurred.png",
                    &screenshot.bytes,
                    screenshot.size.width,
                    screenshot.size.height,
                    ::image::ExtendedColorType::Rgba8,
                );
            }
            Message::ToggleControls => {
                self.controls = !self.controls;
            }
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
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        on_key_release(|key, _| match key {
            Key::Character(key) if key == "o" => Some(Message::ToggleControls),
            Key::Character(key) if key == "p" => Some(Message::Screenshot),
            _ => None,
        })
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

        let background = container(
            image("assets/test.png")
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .padding(1)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::WHITE)),
            ..Default::default()
        });

        column![
            (!self.controls).then(|| container(controls).style(container::dark)),
            stack![
                background,
                self.show.then(|| blur(self.passes, self.offset)
                    .width(Length::Fill)
                    .height(Length::Fill)),
            ]
        ]
        .into()
    }
}
