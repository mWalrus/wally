use smithay::{
    backend::{
        egl::EGLDevice,
        renderer::{damage::OutputDamageTracker, gles::GlesRenderer, ImportDma},
        winit::WinitGraphicsBackend,
    },
    input::keyboard::LedState,
    output::Output,
    reexports::wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    wayland::dmabuf::{DmabufFeedback, DmabufFeedbackBuilder, DmabufGlobal, DmabufState},
};
use tracing::warn;

use crate::state::WallyState;

pub trait Backend {
    const HAS_RELATIVE_MOTION: bool = false;
    const HAS_GESTURES: bool = false;
    fn seat_name(&self) -> String;
    fn reset_buffers(&mut self, output: &Output);
    fn early_import(&mut self, surface: &WlSurface);
    fn update_led_state(&mut self, led_state: LedState);
}

pub struct WinitData {
    pub backend: WinitGraphicsBackend<GlesRenderer>,
    pub damage_tracker: OutputDamageTracker,
    pub dmabuf: BackendDmabufState,
    pub full_redraw: u8,
}

impl Backend for WinitData {
    fn seat_name(&self) -> String {
        String::from("winit")
    }

    fn reset_buffers(&mut self, _output: &Output) {
        self.full_redraw = 4;
    }

    fn early_import(&mut self, _surface: &WlSurface) {}

    fn update_led_state(&mut self, _led_state: LedState) {}
}

pub struct BackendDmabufState {
    pub state: DmabufState,
    pub global: DmabufGlobal,
    pub feedback: Option<DmabufFeedback>,
}

impl BackendDmabufState {
    pub fn new_winit(renderer: &GlesRenderer, display_handle: &DisplayHandle) -> Self {
        // if we failed to build dmabuf feedback we fall back to dmabuf v3
        // Note: egl on Mesa requires either v4 or wl_drm (initialized with bind_wl_display)
        let (state, global, feedback) =
            if let Some(default_feedback) = Self::default_feedback(renderer) {
                let mut dmabuf_state = DmabufState::new();
                let dmabuf_global = dmabuf_state
                    .create_global_with_default_feedback::<WallyState<WinitData>>(
                        display_handle,
                        &default_feedback,
                    );
                (dmabuf_state, dmabuf_global, Some(default_feedback))
            } else {
                let dmabuf_formats = renderer.dmabuf_formats();
                let mut dmabuf_state = DmabufState::new();
                let dmabuf_global = dmabuf_state
                    .create_global::<WallyState<WinitData>>(&display_handle, dmabuf_formats);
                (dmabuf_state, dmabuf_global, None)
            };
        Self {
            state,
            global,
            feedback,
        }
    }
    pub fn default_feedback(renderer: &GlesRenderer) -> Option<DmabufFeedback> {
        let render_node = EGLDevice::device_for_display(renderer.egl_context().display())
            .and_then(|device| device.try_get_render_node());

        match render_node {
            Ok(Some(node)) => {
                let dmabuf_formats = renderer.dmabuf_formats();
                let dmabuf_default_feedback =
                    DmabufFeedbackBuilder::new(node.dev_id(), dmabuf_formats)
                        .build()
                        .unwrap();
                Some(dmabuf_default_feedback)
            }
            Ok(None) => {
                warn!("failed to query render node, dmabuf will use v3");
                None
            }
            Err(err) => {
                warn!(?err, "failed to egl device for display, dmabuf will use v3");
                None
            }
        }
    }
}
