use smithay::{
    backend::renderer::gles::{element::PixelShaderElement, GlesRenderer},
    render_elements,
};

render_elements! {
    pub CustomRenderElement<=GlesRenderer>;
    Pixel=PixelShaderElement,
}
