use smithay::{
    backend::renderer::{
        element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            surface::{self, WaylandSurfaceRenderElement},
            AsRenderElements, Kind,
        },
        gles::{element::PixelShaderElement, GlesRenderer},
        ImportAll, ImportMem, Renderer, Texture,
    },
    input::pointer::CursorImageStatus,
    render_elements,
};

pub struct PointerElement {
    buffer: Option<MemoryRenderBuffer>,
    status: CursorImageStatus,
}

impl Default for PointerElement {
    fn default() -> Self {
        Self {
            buffer: None,
            status: CursorImageStatus::default_named(),
        }
    }
}

impl PointerElement {
    pub fn set_status(&mut self, status: CursorImageStatus) {
        self.status = status;
    }

    pub fn set_buffer(&mut self, buffer: MemoryRenderBuffer) {
        self.buffer = Some(buffer);
    }
}

impl<T: Texture + Clone + Send + 'static, R> AsRenderElements<R> for PointerElement
where
    R: Renderer<TextureId = T> + ImportAll + ImportMem,
{
    type RenderElement = PointerRenderElement<R>;

    fn render_elements<C: From<Self::RenderElement>>(
        &self,
        renderer: &mut R,
        location: smithay::utils::Point<i32, smithay::utils::Physical>,
        scale: smithay::utils::Scale<f64>,
        alpha: f32,
    ) -> Vec<C>
    where
        C: From<PointerRenderElement<R>>,
    {
        match &self.status {
            CursorImageStatus::Hidden => vec![],
            CursorImageStatus::Named(_) => {
                let Some(buffer) = self.buffer.as_ref() else {
                    return vec![];
                };
                vec![PointerRenderElement::<R>::from(
                    MemoryRenderBufferRenderElement::from_buffer(
                        renderer,
                        location.to_f64(),
                        buffer,
                        None,
                        None,
                        None,
                        Kind::Cursor,
                    )
                    .expect("Lost system pointer buffer"),
                )
                .into()]
            }
            CursorImageStatus::Surface(wl_surface) => {
                let elements: Vec<PointerRenderElement<R>> =
                    surface::render_elements_from_surface_tree(
                        renderer,
                        wl_surface,
                        location,
                        scale,
                        alpha,
                        Kind::Cursor,
                    );
                elements.into_iter().map(C::from).collect()
            }
        }
    }
}

// pointer render element
render_elements! {
    pub PointerRenderElement<R> where R: ImportAll + ImportMem;
    Surface=WaylandSurfaceRenderElement<R>,
    Memory=MemoryRenderBufferRenderElement<R>
}

// custom render elements
render_elements! {
    pub CustomRenderElement<=GlesRenderer>;
    Pointer=PointerRenderElement<GlesRenderer>,
    Pixel=PixelShaderElement,
}
