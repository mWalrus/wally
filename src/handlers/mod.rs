mod compositor;
mod xdg_shell;

use crate::backend::Backend;
use crate::WallyState;

//
// Wl Seat
//

use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::Resource;
use smithay::wayland::output::OutputHandler;
use smithay::wayland::selection::data_device::{
    set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
    ServerDndGrabHandler,
};
use smithay::wayland::selection::SelectionHandler;
use smithay::{delegate_data_device, delegate_output, delegate_seat};

impl<BackendData: Backend> SeatHandler for WallyState<BackendData> {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<WallyState<BackendData>> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        image: smithay::input::pointer::CursorImageStatus,
    ) {
        self.cursor_image = image
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let dh = &self.display_handle;
        let client = focused.and_then(|s| dh.get_client(s.id()).ok());
        set_data_device_focus(dh, seat, client);
    }
}

delegate_seat!(@<BackendData: Backend + 'static> WallyState<BackendData>);

//
// Wl Data Device
//

impl<BackendData: Backend> SelectionHandler for WallyState<BackendData> {
    type SelectionUserData = ();
}

impl<BackendData: Backend> DataDeviceHandler for WallyState<BackendData> {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl<BackendData: Backend> ClientDndGrabHandler for WallyState<BackendData> {}
impl<BackendData: Backend> ServerDndGrabHandler for WallyState<BackendData> {}

delegate_data_device!(@<BackendData: Backend + 'static> WallyState<BackendData>);

//
// Wl Output & Xdg Output
//

impl<BackendData: Backend> OutputHandler for WallyState<BackendData> {}
delegate_output!(@<BackendData: Backend + 'static> WallyState<BackendData>);
