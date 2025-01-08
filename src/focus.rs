use smithay::{
    desktop::PopupKind, input::pointer::PointerTarget,
    reexports::wayland_server::protocol::wl_surface::WlSurface, utils::IsAlive,
    wayland::seat::WaylandFocus,
};

use crate::{backend::Backend, elements::window::WindowElement, state::WallyState};

#[derive(Debug, Clone, PartialEq)]
pub enum PointerFocusTarget {
    WlSurface(WlSurface),
    WindowElement(WindowElement), // FIXME: add xwayland surface
}

impl IsAlive for PointerFocusTarget {
    fn alive(&self) -> bool {
        match self {
            PointerFocusTarget::WlSurface(surface) => surface.alive(),
            PointerFocusTarget::WindowElement(window) => window.alive(),
        }
    }
}

impl WaylandFocus for PointerFocusTarget {
    fn wl_surface(
        &self,
    ) -> Option<
        std::borrow::Cow<'_, smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
    > {
        match self {
            PointerFocusTarget::WlSurface(surface) => surface.wl_surface(),
            PointerFocusTarget::WindowElement(window) => window.wl_surface(),
        }
    }

    fn same_client_as(
        &self,
        object_id: &smithay::reexports::wayland_server::backend::ObjectId,
    ) -> bool {
        match self {
            PointerFocusTarget::WlSurface(surface) => surface.same_client_as(object_id),
            PointerFocusTarget::WindowElement(window) => window.same_client_as(object_id),
        }
    }
}

impl From<PointerFocusTarget> for WlSurface {
    fn from(pointer_focus_target: PointerFocusTarget) -> Self {
        pointer_focus_target.wl_surface().unwrap().into_owned()
    }
}

impl From<WlSurface> for PointerFocusTarget {
    fn from(surface: WlSurface) -> Self {
        PointerFocusTarget::WlSurface(surface)
    }
}

impl From<&WlSurface> for PointerFocusTarget {
    fn from(surface: &WlSurface) -> Self {
        PointerFocusTarget::from(surface.clone())
    }
}

impl From<PopupKind> for PointerFocusTarget {
    fn from(popup: PopupKind) -> Self {
        PointerFocusTarget::from(popup.wl_surface())
    }
}

impl<BackendData: Backend> PointerTarget<WallyState<BackendData>> for PointerFocusTarget {
    fn enter(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::enter(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::enter(window, seat, data, event)
            }
        }
    }

    fn motion(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::motion(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::motion(window, seat, data, event)
            }
        }
    }

    fn relative_motion(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::RelativeMotionEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::relative_motion(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::relative_motion(window, seat, data, event)
            }
        }
    }

    fn button(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::button(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::button(window, seat, data, event)
            }
        }
    }

    fn axis(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        frame: smithay::input::pointer::AxisFrame,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::axis(surface, seat, data, frame)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::axis(window, seat, data, frame)
            }
        }
    }

    fn frame(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => PointerTarget::frame(surface, seat, data),
            PointerFocusTarget::WindowElement(window) => PointerTarget::frame(window, seat, data),
        }
    }

    fn gesture_swipe_begin(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::GestureSwipeBeginEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::gesture_swipe_begin(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::gesture_swipe_begin(window, seat, data, event)
            }
        }
    }

    fn gesture_swipe_update(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::GestureSwipeUpdateEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::gesture_swipe_update(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::gesture_swipe_update(window, seat, data, event)
            }
        }
    }

    fn gesture_swipe_end(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::GestureSwipeEndEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::gesture_swipe_end(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::gesture_swipe_end(window, seat, data, event)
            }
        }
    }

    fn gesture_pinch_begin(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::GesturePinchBeginEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::gesture_pinch_begin(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::gesture_pinch_begin(window, seat, data, event)
            }
        }
    }

    fn gesture_pinch_update(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::GesturePinchUpdateEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::gesture_pinch_update(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::gesture_pinch_update(window, seat, data, event)
            }
        }
    }

    fn gesture_pinch_end(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::GesturePinchEndEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::gesture_pinch_end(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::gesture_pinch_end(window, seat, data, event)
            }
        }
    }

    fn gesture_hold_begin(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::GestureHoldBeginEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::gesture_hold_begin(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::gesture_hold_begin(window, seat, data, event)
            }
        }
    }

    fn gesture_hold_end(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        event: &smithay::input::pointer::GestureHoldEndEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::gesture_hold_end(surface, seat, data, event)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::gesture_hold_end(window, seat, data, event)
            }
        }
    }

    fn leave(
        &self,
        seat: &smithay::input::Seat<WallyState<BackendData>>,
        data: &mut WallyState<BackendData>,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        match self {
            PointerFocusTarget::WlSurface(surface) => {
                PointerTarget::leave(surface, seat, data, serial, time)
            }
            PointerFocusTarget::WindowElement(window) => {
                PointerTarget::leave(window, seat, data, serial, time)
            }
        }
    }
}
