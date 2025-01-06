use smithay::{
    backend::{
        egl::EGLDevice,
        renderer::{gles::GlesRenderer, ImportDma},
    },
    input::keyboard::LedState,
    output::Output,
    reexports::wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    wayland::dmabuf::{DmabufFeedback, DmabufFeedbackBuilder, DmabufGlobal, DmabufState},
};
use tracing::warn;

use crate::state::WallyState;

pub mod winit;

pub trait Backend {
    const HAS_RELATIVE_MOTION: bool = false;
    const HAS_GESTURES: bool = false;
    fn seat_name(&self) -> String;
    fn reset_buffers(&mut self, output: &Output);
    fn early_import(&mut self, surface: &WlSurface);
    fn update_led_state(&mut self, led_state: LedState);
}

pub struct BackendDmabufState {
    pub state: DmabufState,
    pub global: DmabufGlobal,
    pub feedback: Option<DmabufFeedback>,
}

impl BackendDmabufState {
    pub fn new(renderer: &GlesRenderer, display_handle: &DisplayHandle) -> Self {
        // if we failed to build dmabuf feedback we fall back to dmabuf v3
        // Note: egl on Mesa requires either v4 or wl_drm (initialized with bind_wl_display)
        let Some(feedback) = Self::default_feedback(renderer) else {
            return Self::new_no_feedback(renderer, display_handle);
        };

        let mut state = DmabufState::new();
        let global =
            state.create_global_with_default_feedback::<WallyState<_>>(display_handle, &feedback);

        Self {
            state,
            global,
            feedback: Some(feedback),
        }
    }

    pub fn new_no_feedback(renderer: &GlesRenderer, display_handle: &DisplayHandle) -> Self {
        let dmabuf_formats = renderer.dmabuf_formats();
        let mut state = DmabufState::new();
        let global = state.create_global::<WallyState<_>>(&display_handle, dmabuf_formats);
        Self {
            state,
            global,
            feedback: None,
        }
    }

    fn default_feedback(renderer: &GlesRenderer) -> Option<DmabufFeedback> {
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
