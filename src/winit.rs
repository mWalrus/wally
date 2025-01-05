use std::time::Duration;

use smithay::{
    backend::{
        renderer::{damage::OutputDamageTracker, gles::GlesRenderer},
        winit::{self, WinitEvent},
    },
    desktop::space::render_output,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Rectangle, Transform},
};

use crate::{
    config::CONFIG,
    render::CustomRenderElement,
    ssd::{self, BorderShader},
    CalloopData, WallyState,
};

pub fn init(
    event_loop: &mut EventLoop<CalloopData>,
    data: &mut CalloopData,
) -> Result<(), Box<dyn std::error::Error>> {
    let display_handle = &mut data.display_handle;
    let state = &mut data.state;

    let (mut backend, winit_event_loop) = winit::init::<GlesRenderer>()?;

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
    let _global = output.create_global::<WallyState>(display_handle);

    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );

    output.set_preferred(mode);

    // TODO: clone and map outputs to each workspace instead
    state.space.map_output(&output, (0, 0));

    let mut output_damage_tracker = OutputDamageTracker::from_output(&output);

    std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);

    event_loop
        .handle()
        .insert_source(winit_event_loop, move |event, _, data| {
            let display = &mut data.display_handle;
            let state = &mut data.state;

            match event {
                WinitEvent::Resized { size, .. } => {
                    output.change_current_state(
                        Some(Mode {
                            size,
                            refresh: 60_000,
                        }),
                        None,
                        None,
                        None,
                    );
                }
                WinitEvent::Input(event) => state.process_input_event(event),
                WinitEvent::Redraw => {
                    let size = backend.window_size();
                    let damage = Rectangle::from_size(size);

                    backend.bind().unwrap();

                    let mut elements = Vec::<CustomRenderElement>::new();

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

                    output_damage_tracker
                        .render_output(backend.renderer(), age, &elements, [0.0, 0.0, 0.0, 1.0])
                        .unwrap();

                    render_output::<_, CustomRenderElement, _, _>(
                        &output,
                        backend.renderer(),
                        1.0,
                        age,
                        [&state.space],
                        elements.as_slice(),
                        &mut output_damage_tracker,
                        [0.0, 0.0, 0.0, 1.0],
                    )
                    .unwrap();

                    backend.submit(Some(&[damage])).unwrap();

                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        )
                    });

                    state.space.refresh();
                    state.popups.cleanup();

                    let _ = display.flush_clients();

                    backend.window().request_redraw();
                }
                WinitEvent::CloseRequested => {
                    state.loop_signal.stop();
                }
                _ => (),
            };
        })?;

    Ok(())
}
