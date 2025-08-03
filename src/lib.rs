use iced_core::widget::tree::{self, Tree};
use iced_core::{Clipboard, Element, Event, Layout, Length, Rectangle, Shell, Size, Widget};
use iced_core::{layout, mouse, renderer};
use iced_widget::Shader;

mod shader;

pub fn blur<Message>(radius: u32) -> Blur<Message> {
    Blur::new(radius)
}

pub struct Blur<Message> {
    shader: Shader<Message, shader::Shader>,
}

impl<Message> Blur<Message> {
    pub fn new(radius: u32) -> Self {
        let shader = iced_widget::shader(shader::Shader::new(radius));

        Self { shader }
    }

    pub fn width(self, width: Length) -> Self {
        Self {
            shader: self.shader.width(width),
        }
    }

    pub fn height(self, height: Length) -> Self {
        Self {
            shader: self.shader.height(height),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Blur<Message>
where
    Renderer: iced_core::Renderer + iced_widget::renderer::wgpu::primitive::Renderer,
{
    fn tag(&self) -> tree::Tag {
        <iced_widget::Shader<Message, shader::Shader> as iced_core::Widget<
            Message,
            Theme,
            Renderer,
        >>::tag(&self.shader)
    }

    fn state(&self) -> tree::State {
        <iced_widget::Shader<Message, shader::Shader> as iced_core::Widget<
            Message,
            Theme,
            Renderer,
        >>::state(&self.shader)
    }

    fn size(&self) -> Size<Length> {
        <iced_widget::Shader<Message, shader::Shader> as iced_core::Widget<
            Message,
            Theme,
            Renderer,
        >>::size(&self.shader)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        <iced_widget::Shader<Message, shader::Shader> as iced_core::Widget<
            Message,
            Theme,
            Renderer,
        >>::layout(&self.shader, tree, renderer, limits)
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        <iced_widget::Shader<Message, shader::Shader> as iced_core::Widget<
            Message,
            Theme,
            Renderer,
        >>::update(
            &mut self.shader,
            state,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        <iced_widget::Shader<Message, shader::Shader> as iced_core::Widget<
            Message,
            Theme,
            Renderer,
        >>::mouse_interaction(&self.shader, state, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.shader
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }
}

impl<'a, Message, Theme, Renderer> From<Blur<Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a,
    Renderer: iced_core::Renderer + iced_widget::renderer::wgpu::primitive::Renderer,
{
    fn from(blur: Blur<Message>) -> Self {
        Element::new(blur)
    }
}
