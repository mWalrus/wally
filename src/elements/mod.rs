use pointer::PointerRenderElement;
use smithay::{
    backend::renderer::gles::{element::PixelShaderElement, GlesRenderer},
    render_elements,
};

pub mod border;
pub mod pointer;

// custom render elements
render_elements! {
    pub CustomRenderElement<=GlesRenderer>;
    Pointer=PointerRenderElement<GlesRenderer>,
    Pixel=PixelShaderElement,
}
