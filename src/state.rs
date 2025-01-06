use std::{
    process::Command,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use smithay::{
    desktop::{PopupManager, Space, Window, WindowSurfaceType},
    input::{
        pointer::{CursorImageAttributes, CursorImageStatus, PointerHandle},
        Seat, SeatState,
    },
    reexports::{
        calloop::{generic::Generic, Interest, LoopHandle, Mode, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle,
        },
    },
    utils::{Clock, IsAlive, Logical, Monotonic, Physical, Point, Scale},
    wayland::{
        compositor::{self, CompositorClientState, CompositorState},
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::xdg::{decoration::XdgDecorationState, XdgShellState},
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

use crate::{backend::Backend, monitor::Monitor, types::keybind::Action};

#[derive(Debug)]
pub struct WallyState<BackendData: Backend + 'static> {
    pub running: AtomicBool,
    pub backend_data: BackendData,
    pub clock: Clock<Monotonic>,
    pub start_time: std::time::Instant,
    pub socket_name: String,
    pub display_handle: DisplayHandle,

    pub monitors: Vec<Monitor>,
    pub space: Space<Window>,

    // Smithay State
    pub cursor_status: CursorImageStatus,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub xdg_decoration_state: XdgDecorationState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<WallyState<BackendData>>,
    pub data_device_state: DataDeviceState,
    pub popups: PopupManager,

    pub seat: Seat<WallyState<BackendData>>,
    pub pointer: PointerHandle<WallyState<BackendData>>,
}

impl<BackendData: Backend> WallyState<BackendData> {
    pub fn new(
        display: Display<WallyState<BackendData>>,
        handle: LoopHandle<'static, WallyState<BackendData>>,
        backend_data: BackendData,
    ) -> Self {
        let start_time = std::time::Instant::now();

        let display_handle = display.handle();

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let popups = PopupManager::default();

        let seat_name = backend_data.seat_name();
        let mut seat: Seat<Self> = seat_state.new_wl_seat(&display_handle, seat_name);

        // Notify clients that we have a keyboard, for the sake of the example we assume that keyboard is always present.
        // You may want to track keyboard hot-plug in real compositor.
        seat.add_keyboard(Default::default(), 200, 25).unwrap();

        // Notify clients that we have a pointer (mouse)
        // Here we assume that there is always pointer plugged in
        let pointer = seat.add_pointer();

        // A space represents a two-dimensional plane. Windows and Outputs can be mapped onto it.
        //
        // Windows get a position and stacking order through mapping.
        // Outputs become views of a part of the Space and can be rendered via Space::render_output.
        let space = Space::default();

        let socket_name = Self::init_wayland_listener(display, handle);

        Self {
            running: AtomicBool::new(true),
            backend_data,
            clock: Clock::new(),
            start_time,
            display_handle,

            monitors: Vec::new(),
            space,
            socket_name,

            cursor_status: CursorImageStatus::default_named(),
            compositor_state,
            xdg_shell_state,
            xdg_decoration_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            popups,
            seat,
            pointer,
        }
    }

    fn init_wayland_listener(
        display: Display<WallyState<BackendData>>,
        loop_handle: LoopHandle<'static, WallyState<BackendData>>,
    ) -> String {
        // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let listening_socket = ListeningSocketSource::new_auto().unwrap();

        // Get the name of the listening socket.
        // Clients will connect to this socket.
        let socket_name = listening_socket
            .socket_name()
            .to_string_lossy()
            .into_owned();

        loop_handle
            .insert_source(listening_socket, move |client_stream, _, state| {
                // Inside the callback, you should insert the client into the display.
                //
                // You may also associate some data with the client when inserting the client.
                state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::new()))
                    .unwrap();
            })
            .expect("Failed to init the wayland event source.");

        // You also need to add the display itself to the event loop, so that client events will be processed by wayland-server.
        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, state| {
                    // Safety: we don't drop the display
                    unsafe {
                        display.get_mut().dispatch_clients(state).unwrap();
                    }
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

        socket_name
    }

    pub fn get_cursor_data(&mut self, scale: Scale<f64>) -> (bool, Point<i32, Physical>) {
        if let CursorImageStatus::Surface(ref surface) = self.cursor_status {
            if !surface.alive() {
                self.cursor_status = CursorImageStatus::default_named();
            }
        }

        let cursor_pos = self.pointer.current_location();
        let cursor_location = if let CursorImageStatus::Surface(ref surface) = self.cursor_status {
            // the 'hotspot' is the part of the cursor image where the tip of the arrow
            // is situated.
            let hotspot = compositor::with_states(surface, |states| {
                states
                    .data_map
                    .get::<Mutex<CursorImageAttributes>>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .hotspot
            });
            cursor_pos - hotspot.to_f64()
        } else {
            cursor_pos
        };

        let cursor_location = cursor_location.to_physical(scale).to_i32_round();

        let cursor_visible = !matches!(self.cursor_status, CursorImageStatus::Surface(_));

        (cursor_visible, cursor_location)
    }

    pub fn add_monitor(&mut self, monitor: Monitor) {
        self.monitors.push(monitor);
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Spawn(command) => {
                Command::new(command)
                    .env("WAYLAND_DISPLAY", &self.socket_name) // FIXME: xwayland DISPLAY
                    .spawn()
                    .ok();
            }
            _ => {}
        }
    }

    pub fn surface_under(
        &self,
        pos: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        self.space
            .element_under(pos)
            .and_then(|(window, location)| {
                window
                    .surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)
                    .map(|(s, p)| (s, (p + location).to_f64()))
            })
    }
}

#[derive(Debug)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            compositor_state: CompositorClientState::default(),
        }
    }
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
