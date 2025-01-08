use std::{borrow::Cow, time::Duration};

use smithay::{
    backend::renderer::{
        element::{surface::WaylandSurfaceRenderElement, AsRenderElements},
        gles::{element::PixelShaderElement, GlesRenderer},
    },
    desktop::{space::SpaceElement, Window, WindowSurface, WindowSurfaceType},
    output::Output,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    render_elements,
    utils::{IsAlive, Logical, Physical, Point, Rectangle},
    wayland::{compositor::SurfaceData, seat::WaylandFocus},
};

use crate::config::CONFIG;

use super::border::BorderShader;

#[derive(Debug, Clone, PartialEq)]
pub struct WindowElement(pub Window);

impl WindowElement {
    pub fn surface_under(
        &self,
        location: Point<f64, Logical>,
        surface_type: WindowSurfaceType,
    ) -> Option<(WlSurface, Point<i32, Logical>)> {
        let offset = Point::from((CONFIG.border_thickness, CONFIG.border_thickness));

        let surface_under = self
            .0
            .surface_under(location - offset.to_f64(), surface_type);

        surface_under
    }

    #[inline]
    pub fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        self.0.wl_surface()
    }

    #[inline]
    pub fn set_activated(&self, activated: bool) {
        self.0.set_activated(activated);
    }

    pub fn surface_matches(&self, other: &WlSurface) -> bool {
        self.0
            .wl_surface()
            .map(|surface| &*surface == other)
            .unwrap_or(false)
    }

    pub fn send_frame<T, F>(
        &self,
        output: &Output,
        time: T,
        throttle: Option<Duration>,
        primary_scan_out_output: F,
    ) where
        T: Into<Duration>,
        F: FnMut(&WlSurface, &SurfaceData) -> Option<Output> + Copy,
    {
        self.0
            .send_frame(output, time, throttle, primary_scan_out_output);
    }

    pub fn send_pending_configure(&self) {
        match self.0.underlying_surface() {
            WindowSurface::Wayland(toplevel) => {
                toplevel.send_pending_configure();
            }
        }
    }

    pub fn send_configure(&self) {
        match self.0.underlying_surface() {
            WindowSurface::Wayland(toplevel) => {
                toplevel.send_configure();
            }
        }
    }
}

impl IsAlive for WindowElement {
    fn alive(&self) -> bool {
        self.0.alive()
    }
}

// make element mappable onto a space
impl SpaceElement for WindowElement {
    fn geometry(&self) -> Rectangle<i32, Logical> {
        let mut geometry = SpaceElement::geometry(&self.0);
        let border_size = CONFIG.border_thickness * 2;
        geometry.size += (border_size, border_size).into();
        geometry
    }
    fn bbox(&self) -> Rectangle<i32, Logical> {
        let mut bounding_box = SpaceElement::bbox(&self.0);
        let border_size = CONFIG.border_thickness * 2;
        bounding_box.size += (border_size, border_size).into();
        bounding_box
    }

    fn is_in_input_region(&self, point: &Point<f64, Logical>) -> bool {
        // FIXME: we should probably include borders in this check
        SpaceElement::is_in_input_region(&self.0, point)
    }

    fn z_index(&self) -> u8 {
        SpaceElement::z_index(&self.0)
    }

    fn set_activate(&self, activated: bool) {
        SpaceElement::set_activate(&self.0, activated);
    }

    fn output_enter(&self, output: &Output, overlap: Rectangle<i32, Logical>) {
        SpaceElement::output_enter(&self.0, output, overlap);
    }

    fn output_leave(&self, output: &Output) {
        SpaceElement::output_leave(&self.0, output);
    }

    fn refresh(&self) {
        SpaceElement::refresh(&self.0);
    }
}

render_elements! {
    pub WindowRenderElement<=GlesRenderer>;
    Window=WaylandSurfaceRenderElement<GlesRenderer>,
    Border=PixelShaderElement
}

impl AsRenderElements<GlesRenderer> for WindowElement {
    type RenderElement = WindowRenderElement;

    fn render_elements<C: From<Self::RenderElement>>(
        &self,
        renderer: &mut GlesRenderer,
        location: Point<i32, Physical>,
        scale: smithay::utils::Scale<f64>,
        alpha: f32,
    ) -> Vec<C> {
        // get the inner window's bounding box, not the wrapping `WindowElement`
        let window_bounding_box = SpaceElement::bbox(&self.0);

        if !window_bounding_box.is_empty() {
            let window_geometry = SpaceElement::geometry(&self.0);

            let border_thickness = CONFIG.border_thickness;

            let loc: Point<i32, Logical> =
                (location.x - border_thickness, location.y - border_thickness).into();
            let size = window_geometry.size + (border_thickness * 2, border_thickness * 2).into();

            let border_geometry = Rectangle::new(loc, size);

            let border = BorderShader::element(
                renderer,
                border_geometry,
                CONFIG.border_color_focused,
                border_thickness,
            );

            let mut vec: Vec<WindowRenderElement> = vec![border.into()];

            let window_elements =
                AsRenderElements::render_elements(&self.0, renderer, location, scale, alpha);

            vec.extend(window_elements);

            return vec.into_iter().map(C::from).collect();
        }

        Vec::new()
    }
}
