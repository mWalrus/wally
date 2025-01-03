use smithay::{
    backend::renderer::{
        element::{
            solid::{SolidColorBuffer, SolidColorRenderElement},
            AsRenderElements, Kind,
        },
        Renderer,
    },
    utils::{Physical, Point, Scale},
};

#[derive(Clone, Debug)]
pub struct Border {
    size: u8,
    is_focused: bool,
    color_focused: SolidColorBuffer,
    color_unfocused: SolidColorBuffer,
}

impl Border {
    pub fn new(size: u8, color_focused: [f32; 4], color_unfocused: [f32; 4]) -> Self {
        let mut focused = SolidColorBuffer::default();
        let mut unfocused = SolidColorBuffer::default();

        focused.set_color(color_focused);
        unfocused.set_color(color_unfocused);

        Self {
            size,
            is_focused: false,
            color_focused: focused,
            color_unfocused: unfocused,
        }
    }

    pub fn set_focus(&mut self) {
        self.is_focused = true;
    }

    pub fn remove_focus(&mut self) {
        self.is_focused = false;
    }
}

impl<R: Renderer> AsRenderElements<R> for Border {
    type RenderElement = SolidColorRenderElement;

    fn render_elements<C: From<Self::RenderElement>>(
        &self,
        _renderer: &mut R,
        location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<C> {
        let buffer = if self.is_focused {
            &self.color_focused
        } else {
            &self.color_unfocused
        };

        let offset = self.size as i32;
        let location = Point::from((location.x - offset, location.y - offset));

        vec![SolidColorRenderElement::from_buffer(
            buffer,
            location,
            scale,
            alpha,
            Kind::Unspecified,
        )
        .into()]
    }
}
