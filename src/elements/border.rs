use anyhow::Result;
use smithay::{
    backend::renderer::{
        element::Kind,
        gles::{element::PixelShaderElement, GlesPixelProgram, GlesRenderer, Uniform, UniformName},
    },
    utils::{Logical, Rectangle},
};

const BORDER_SHADER: &str = include_str!("../shaders/border.frag");

pub struct BorderShader(pub GlesPixelProgram);

impl BorderShader {
    pub fn element(
        renderer: &GlesRenderer,
        geometry: Rectangle<i32, Logical>,
        color: u32,
        thickness: u8,
    ) -> PixelShaderElement {
        let program = renderer
            .egl_context()
            .user_data()
            .get::<BorderShader>()
            .unwrap()
            .0
            .clone();

        let point = geometry.size.to_point();

        let red = color >> 16 & 255;
        let green = color >> 8 & 255;
        let blue = color & 255;

        PixelShaderElement::new(
            program,
            geometry,
            None,
            1.0,
            vec![
                Uniform::new("u_resolution", (point.x as f32, point.y as f32)),
                Uniform::new("border_color", (red as f32, green as f32, blue as f32)),
                Uniform::new("border_thickness", thickness as f32),
            ],
            Kind::Unspecified,
        )
    }
}

pub fn compile_shaders(renderer: &mut GlesRenderer) -> Result<()> {
    let border_shader = renderer.compile_custom_pixel_shader(
        BORDER_SHADER,
        &[
            UniformName::new(
                "u_resolution",
                smithay::backend::renderer::gles::UniformType::_2f,
            ),
            UniformName::new(
                "border_color",
                smithay::backend::renderer::gles::UniformType::_3f,
            ),
            UniformName::new(
                "border_thickness",
                smithay::backend::renderer::gles::UniformType::_1f,
            ),
        ],
    )?;

    renderer
        .egl_context()
        .user_data()
        .insert_if_missing(|| BorderShader(border_shader));

    Ok(())
}
