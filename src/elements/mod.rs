use pointer::PointerRenderElement;
use smithay::{backend::renderer::gles::GlesRenderer, render_elements};
use window::WindowRenderElement;

pub mod border;
pub mod pointer;
pub mod window;

// custom render elements
render_elements! {
    pub CustomRenderElement<=GlesRenderer>;
    Pointer=PointerRenderElement<GlesRenderer>,
    Window=WindowRenderElement,
}
