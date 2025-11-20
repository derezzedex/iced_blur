use iced::{Element, Length};

mod shader;

pub fn blur(passes: u32, offset: f32) -> Blur {
    Blur::new(passes, offset)
}

pub struct Blur {
    passes: u32,
    offset: f32,
    width: Length,
    height: Length,
}

impl Blur {
    pub fn new(passes: u32, offset: f32) -> Self {
        Self {
            passes,
            offset,
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    pub fn width(self, width: Length) -> Self {
        Self { width, ..self }
    }

    pub fn height(self, height: Length) -> Self {
        Self { height, ..self }
    }
}

impl<'a, Message, Theme, Renderer> From<Blur> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + iced::widget::shader::Renderer,
{
    fn from(blur: Blur) -> Self {
        let blur = iced::widget::shader(shader::Shader::new(blur.passes, blur.offset))
            .width(blur.width)
            .height(blur.height);
        Element::new(blur)
    }
}
