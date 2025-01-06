use std::{
    sync::{atomic::Ordering, Mutex},
    time::Duration,
};

use crate::{
    config::CONFIG,
    render::{CustomRenderElement, PointerElement},
    ssd::{self, BorderShader},
    WallyState,
};
use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        renderer::{
            damage::{Error as OutputDamageTrackerError, OutputDamageTracker},
            element::AsRenderElements,
            gles::GlesRenderer,
            ImportDma, ImportEgl, ImportMemWl,
        },
        winit::{self, WinitEvent, WinitGraphicsBackend},
        SwapBuffersError,
    },
    delegate_dmabuf,
    desktop::space::render_output,
    input::{
        keyboard::LedState,
        pointer::{CursorImageAttributes, CursorImageStatus},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::EventLoop,
        wayland_server::{protocol::wl_surface::WlSurface, Display},
        winit::platform::pump_events::PumpStatus,
    },
    utils::{IsAlive, Rectangle, Scale, Transform},
    wayland::{
        compositor,
        dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
    },
};
use tracing::{error, info, warn};

use super::{Backend, BackendDmabufState};

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

impl DmabufHandler for WallyState<WinitData> {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.backend_data.dmabuf.state
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        if self
            .backend_data
            .backend
            .renderer()
            .import_dmabuf(&dmabuf, None)
            .is_ok()
        {
            let _ = notifier.successful::<WallyState<WinitData>>();
        } else {
            notifier.failed();
        }
    }
}

delegate_dmabuf!(WallyState<WinitData>);

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = EventLoop::try_new()?;
    let display = Display::new()?;

    let mut display_handle = display.handle();

    let (mut backend, mut winit_event_loop) = winit::init::<GlesRenderer>()?;

    ssd::compile_shaders(backend.renderer())?;

    let mode = Mode {
        size: backend.window_size(),
        refresh: 60_000,
    };

    let output_properties = PhysicalProperties {
        size: (0, 0).into(),
        subpixel: Subpixel::Unknown,
        make: "Wally".into(),
        model: "Winit".into(),
    };

    let output = Output::new("winit".to_string(), output_properties);

    // Clients can access the global objects to get the physical properties and output state.
    let _global = output.create_global::<WallyState<WinitData>>(&display_handle);

    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );

    output.set_preferred(mode);

    let dmabuf = BackendDmabufState::new(backend.renderer(), &display_handle);

    if backend.renderer().bind_wl_display(&display_handle).is_ok() {
        info!("EGL hardware-acceleration enabled");
    }

    let winit_data = {
        let damage_tracker = OutputDamageTracker::from_output(&output);

        WinitData {
            backend,
            damage_tracker,
            dmabuf,
            full_redraw: 0,
        }
    };

    let mut state = WallyState::new(display, event_loop.handle(), winit_data);

    // update the global shared memory formats to the
    // smh formats supported by the backend's renderer
    state
        .shm_state
        .update_formats(state.backend_data.backend.renderer().shm_formats());

    // TODO: clone and map outputs to each workspace instead
    state.space.map_output(&output, (0, 0));

    let mut output_damage_tracker = OutputDamageTracker::from_output(&output);

    std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);

    let mut pointer_element = PointerElement::default();

    while state.running.load(Ordering::SeqCst) {
        let status = winit_event_loop.dispatch_new_events(|event| match event {
            WinitEvent::Resized { size, .. } => {
                let output = state.space.outputs().next().unwrap().clone();
                state.space.map_output(&output, (0, 0));

                let mode = Mode {
                    size,
                    refresh: 60_000,
                };

                output.change_current_state(Some(mode), None, None, None);
                output.set_preferred(mode);
            }
            WinitEvent::Input(event) => state.process_input_event(event),
            _ => (),
        });

        if let PumpStatus::Exit(_) = status {
            state.running.store(false, Ordering::SeqCst);
            break;
        }

        draw(
            &mut state,
            &mut pointer_element,
            &mut output_damage_tracker,
            &output,
        );

        // dispatch all pending events accumulated during the draw routine
        // so that they will be processed during the next cycle of the event loop
        if event_loop
            .dispatch(Some(Duration::from_millis(1)), &mut state)
            .is_err()
        {
            // if we fail we signal for the event loop to halt
            state.running.store(false, Ordering::SeqCst);
        } else {
            // otherwise we refresh the space, cleaning up some internals and update client state...
            state.space.refresh();
            // ...as well as clean up some internal popup resources...
            state.popups.cleanup();

            // ...and lastly we flush outgoing buffers into their respective sockets
            display_handle.flush_clients().unwrap();
        }
    }

    Ok(())
}

fn draw(
    state: &mut WallyState<WinitData>,
    pointer: &mut PointerElement,
    damage_tracker: &mut OutputDamageTracker,
    output: &Output,
) {
    let backend = &mut state.backend_data.backend;

    let size = backend.window_size();
    let damage = Rectangle::from_size(size);

    let mut elements = Vec::<CustomRenderElement>::new();

    let current_output_fractional_scale = Scale::from(output.current_scale().fractional_scale());

    let render_result = backend.bind().and_then(|_| {
        if let CursorImageStatus::Surface(ref surface) = state.cursor_status {
            if !surface.alive() {
                state.cursor_status = CursorImageStatus::default_named();
            }
        }

        let cursor_visible = !matches!(state.cursor_status, CursorImageStatus::Surface(_));

        pointer.set_status(state.cursor_status.clone());

        let cursor_hotspot = if let CursorImageStatus::Surface(ref surface) = state.cursor_status {
            compositor::with_states(surface, |states| {
                states
                    .data_map
                    .get::<Mutex<CursorImageAttributes>>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .hotspot
            })
        } else {
            (0, 0).into()
        };

        let cursor_pos = state.pointer.current_location();

        elements.extend(
            pointer.render_elements(
                backend.renderer(),
                (cursor_pos - cursor_hotspot.to_f64())
                    .to_physical(current_output_fractional_scale)
                    .to_i32_round(),
                current_output_fractional_scale,
                1.0,
            ),
        );

        let border_thickness = CONFIG.border_thickness as i32;

        for window in state.space.elements() {
            let Some(mut geometry) = state.space.element_geometry(window) else {
                continue;
            };

            geometry.size += (border_thickness * 2, border_thickness * 2).into();

            geometry.loc -= (border_thickness, border_thickness).into();

            elements.push(CustomRenderElement::from(BorderShader::element(
                backend.renderer(),
                geometry,
                CONFIG.border_color_focused,
                CONFIG.border_thickness,
            )));
        }

        let age = backend.buffer_age().unwrap_or(0);

        damage_tracker
            .render_output(backend.renderer(), age, &elements, [0.0, 0.0, 0.0, 1.0])
            .unwrap();

        render_output::<_, CustomRenderElement, _, _>(
            &output,
            backend.renderer(),
            1.0,
            age,
            [&state.space],
            elements.as_slice(),
            damage_tracker,
            [0.0, 0.0, 0.0, 1.0],
        )
        .map_err(|err| match err {
            OutputDamageTrackerError::Rendering(err) => err.into(),
            _ => unreachable!(),
        })
    });

    match render_result {
        Ok(render_output_result) => {
            if let Some(damage) = render_output_result.damage {
                if let Err(err) = backend.submit(Some(damage)) {
                    warn!("Failed to submit damage buffer: {err}");
                }
            }

            state.space.elements().for_each(|window| {
                window.send_frame(
                    &output,
                    state.start_time.elapsed(),
                    Some(Duration::ZERO),
                    |_, _| Some(output.clone()),
                )
            });
        }
        Err(SwapBuffersError::ContextLost(err)) => {
            error!("Critical rendering error: {err}");
            state.running.store(false, Ordering::SeqCst);
        }
        Err(err) => warn!("Rendering error: {err}"),
    }
}
