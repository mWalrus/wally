use std::{
    borrow::Cow,
    cell::{RefCell, RefMut},
    time::Duration,
};

use smithay::{
    backend::renderer::{
        element::{surface::WaylandSurfaceRenderElement, AsRenderElements},
        gles::{element::PixelShaderElement, GlesRenderer},
    },
    desktop::{space::SpaceElement, Window, WindowSurface, WindowSurfaceType},
    input::{
        pointer::{
            GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
            GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
            GestureSwipeEndEvent, GestureSwipeUpdateEvent, MotionEvent, PointerTarget,
            RelativeMotionEvent,
        },
        Seat,
    },
    output::Output,
    reexports::wayland_server::{backend::ObjectId, protocol::wl_surface::WlSurface, Resource},
    render_elements,
    utils::{
        user_data::UserDataMap, IsAlive, Logical, Physical, Point, Rectangle, Serial,
        SERIAL_COUNTER,
    },
    wayland::{compositor::SurfaceData, seat::WaylandFocus},
};

use crate::{backend::Backend, config::CONFIG, focus::PointerFocusTarget, state::WallyState};

use super::border::BorderShader;

#[derive(Debug, Clone, PartialEq)]
pub struct WindowElement(pub Window);

pub struct WindowState {
    is_focused: bool,
}

impl WindowElement {
    pub fn surface_under(
        &self,
        _location: Point<f64, Logical>,
        _surface_type: WindowSurfaceType,
    ) -> Option<(PointerFocusTarget, Point<i32, Logical>)> {
        // FIXME: always returning this element prevents me from rendering
        //        the surface cursors for the current window
        Some((
            PointerFocusTarget::WindowElement(self.clone()),
            Point::default(),
        ))
        // self.0
        //     .surface_under(location, surface_type)
        //     .map(|(surface, loc)| (PointerFocusTarget::WlSurface(surface), loc))
    }

    pub fn window_state(&self) -> RefMut<'_, WindowState> {
        // NOTE: we set focus to true when spawning a new window state
        //       since we will want the window to be focused on creation
        self.user_data()
            .insert_if_missing(|| RefCell::new(WindowState { is_focused: true }));

        self.user_data()
            .get::<RefCell<WindowState>>()
            .unwrap()
            .borrow_mut()
    }

    #[inline]
    pub fn user_data(&self) -> &UserDataMap {
        self.0.user_data()
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

    pub fn on_commit(&self) {
        self.0.on_commit();
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

    pub fn same_client_as(&self, object_id: &ObjectId) -> bool {
        self.wl_surface()
            .map(|s| s.id().same_client_as(object_id))
            .unwrap_or(false)
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

        if window_bounding_box.is_empty() {
            return Vec::new();
        }

        let border_thickness = CONFIG.border_thickness;

        let border_geometry = {
            let window_geometry = SpaceElement::geometry(&self.0);
            let loc: Point<i32, Logical> =
                (location.x - border_thickness, location.y - border_thickness).into();
            let size = window_geometry.size + (border_thickness * 2, border_thickness * 2).into();

            Rectangle::new(loc, size)
        };

        let color = {
            let state = self.window_state();
            if state.is_focused {
                CONFIG.border_color_focused
            } else {
                CONFIG.border_color_unfocused
            }
        };

        let border = BorderShader::element(renderer, border_geometry, color, border_thickness);

        let mut vec: Vec<WindowRenderElement> = vec![border.into()];

        let window_elements =
            AsRenderElements::render_elements(&self.0, renderer, location, scale, alpha);

        vec.extend(window_elements);

        return vec.into_iter().map(C::from).collect();
    }
}

impl<BackendData: Backend> PointerTarget<WallyState<BackendData>> for WindowElement {
    fn enter(
        &self,
        seat: &Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        _event: &MotionEvent,
    ) {
        let serial = SERIAL_COUNTER.next_serial();
        let Some(surface) = self.wl_surface() else {
            return;
        };

        let Some(keyboard) = seat.get_keyboard() else {
            return;
        };

        keyboard.set_focus(data, Some((*surface).clone()), serial);

        let mut window_state = self.window_state();
        window_state.is_focused = true;

        // TODO: figure out what this does
        self.set_activated(true);

        data.space
            .elements()
            .for_each(|window| window.send_pending_configure());
    }

    fn motion(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &MotionEvent,
    ) {
    }

    fn relative_motion(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &RelativeMotionEvent,
    ) {
    }

    fn button(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &smithay::input::pointer::ButtonEvent,
    ) {
    }

    fn axis(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _frame: smithay::input::pointer::AxisFrame,
    ) {
    }

    fn frame(&self, _seat: &Seat<WallyState<BackendData>>, _data: &mut WallyState<BackendData>) {}

    fn gesture_swipe_begin(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &GestureSwipeBeginEvent,
    ) {
    }

    fn gesture_swipe_update(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &GestureSwipeUpdateEvent,
    ) {
    }

    fn gesture_swipe_end(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &GestureSwipeEndEvent,
    ) {
    }

    fn gesture_pinch_begin(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &GesturePinchBeginEvent,
    ) {
    }

    fn gesture_pinch_update(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &GesturePinchUpdateEvent,
    ) {
    }

    fn gesture_pinch_end(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &GesturePinchEndEvent,
    ) {
    }

    fn gesture_hold_begin(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &GestureHoldBeginEvent,
    ) {
    }

    fn gesture_hold_end(
        &self,
        _seat: &Seat<WallyState<BackendData>>,
        _data: &mut WallyState<BackendData>,
        _event: &GestureHoldEndEvent,
    ) {
    }

    fn leave(
        &self,
        seat: &Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        serial: Serial,
        _time: u32,
    ) {
        let Some(keyboard) = seat.get_keyboard() else {
            return;
        };

        keyboard.set_focus(data, None, serial);

        let mut state = self.window_state();
        state.is_focused = false;

        self.set_activated(false);

        data.space
            .elements()
            .for_each(|window| window.send_pending_configure());
    }
}
