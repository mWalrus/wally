use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent,
    },
    input::{
        keyboard::FilterResult,
        pointer::{AxisFrame, ButtonEvent, MotionEvent, RelativeMotionEvent},
    },
    utils::{Logical, Point, SERIAL_COUNTER},
};

use crate::{backend::Backend, config::CONFIG, state::WallyState, types::keybind::Keybind};

impl<BackendData: Backend> WallyState<BackendData> {
    pub fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
            InputEvent::Keyboard { event, .. } => {
                let serial = SERIAL_COUNTER.next_serial();
                let time = Event::time_msec(&event);

                let keyboard = self.seat.get_keyboard().unwrap();

                if let Some(action) = keyboard.input(
                    self,
                    event.key_code(),
                    event.state(),
                    serial,
                    time,
                    |_state, modifiers_state, keysym_handle| {
                        if let KeyState::Pressed = event.state() {
                            // we should be able to get away with this since we wont have combo-binds
                            let raw_syms = keysym_handle.raw_syms();
                            let keysym = raw_syms.into_iter().next().unwrap();
                            let keybind = Keybind::new(modifiers_state, keysym);

                            if let Some(action) = CONFIG.keybinds.get(&keybind) {
                                return FilterResult::Intercept(action.clone());
                            }
                        }
                        FilterResult::Forward
                    },
                ) {
                    tracing::info!(action = ?action, "Got action!");
                    self.handle_action(action);
                }
            }
            InputEvent::PointerMotion { event } => {
                // TODO:
                //     - Check whether pointer is "locked"
                //         - Handle pointer lock
                //     - Check whether pointer is "confined" and to what region
                //         - Handle pointer confinement

                let pointer_location = self.pointer.current_location();
                let under = self.surface_under(pointer_location);

                let serial = SERIAL_COUNTER.next_serial();

                let pointer = self.pointer.clone();
                pointer.relative_motion(
                    self,
                    under.clone(),
                    &RelativeMotionEvent {
                        delta: event.delta(),
                        delta_unaccel: event.delta_unaccel(),
                        utime: event.time(),
                    },
                );

                let pointer_location = self.clamp_coords(pointer_location);

                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: pointer_location,
                        serial,
                        time: event.time_msec(),
                    },
                );

                pointer.frame(self);
            }
            InputEvent::PointerMotionAbsolute { event, .. } => {
                let output = self.space.outputs().next().unwrap();

                let output_geo = self.space.output_geometry(output).unwrap();

                let pos = event.position_transformed(output_geo.size) + output_geo.loc.to_f64();

                let serial = SERIAL_COUNTER.next_serial();

                let pointer = self.pointer.clone();

                let under = self.surface_under(pos);

                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: pos,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerButton { event, .. } => {
                let pointer = self.pointer.clone();
                let keyboard = self.seat.get_keyboard().unwrap();

                let serial = SERIAL_COUNTER.next_serial();

                let button = event.button_code();

                let button_state = event.state();

                if ButtonState::Pressed == button_state && !pointer.is_grabbed() {
                    if let Some((window, _loc)) = self
                        .space
                        .element_under(pointer.current_location())
                        .map(|(w, l)| (w.clone(), l))
                    {
                        self.space.raise_element(&window, true);

                        self.space.elements().for_each(|window| {
                            window.send_pending_configure();
                        });
                    } else {
                        self.space.elements().for_each(|window| {
                            window.set_activated(false);
                            window.send_pending_configure();
                        });
                        keyboard.set_focus(self, None, serial);
                    }
                };

                pointer.button(
                    self,
                    &ButtonEvent {
                        button,
                        state: button_state,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerAxis { event, .. } => {
                let source = event.source();

                let horizontal_amount = event.amount(Axis::Horizontal).unwrap_or_else(|| {
                    event.amount_v120(Axis::Horizontal).unwrap_or(0.0) * 15.0 / 120.
                });
                let vertical_amount = event.amount(Axis::Vertical).unwrap_or_else(|| {
                    event.amount_v120(Axis::Vertical).unwrap_or(0.0) * 15.0 / 120.
                });
                let horizontal_amount_discrete = event.amount_v120(Axis::Horizontal);
                let vertical_amount_discrete = event.amount_v120(Axis::Vertical);

                let mut frame = AxisFrame::new(event.time_msec()).source(source);
                if horizontal_amount != 0.0 {
                    frame = frame.value(Axis::Horizontal, horizontal_amount);
                    if let Some(discrete) = horizontal_amount_discrete {
                        frame = frame.v120(Axis::Horizontal, discrete as i32);
                    }
                }
                if vertical_amount != 0.0 {
                    frame = frame.value(Axis::Vertical, vertical_amount);
                    if let Some(discrete) = vertical_amount_discrete {
                        frame = frame.v120(Axis::Vertical, discrete as i32);
                    }
                }

                if source == AxisSource::Finger {
                    if event.amount(Axis::Horizontal) == Some(0.0) {
                        frame = frame.stop(Axis::Horizontal);
                    }
                    if event.amount(Axis::Vertical) == Some(0.0) {
                        frame = frame.stop(Axis::Vertical);
                    }
                }

                let pointer = self.seat.get_pointer().unwrap();
                pointer.axis(self, frame);
                pointer.frame(self);
            }
            _ => {}
        }
    }

    /// Adjust a coordinate point to within the total space of all outputs
    fn clamp_coords(&self, pos: Point<f64, Logical>) -> Point<f64, Logical> {
        if self.space.outputs().next().is_none() {
            return pos;
        }

        let (pos_x, pos_y) = pos.into();

        let max_x = self.space.outputs().fold(0, |total_width, output| {
            total_width + self.space.output_geometry(output).unwrap().size.w
        });

        let clamped_x = pos_x.clamp(0.0, max_x as f64);

        let max_y = self
            .space
            .outputs()
            .find(|output| {
                let output_geometry = self.space.output_geometry(output).unwrap();
                output_geometry.contains((clamped_x as i32, 0))
            })
            .map(|output| self.space.output_geometry(output).unwrap().size.h);

        let Some(max_y) = max_y else {
            return (clamped_x, pos_y).into();
        };

        let clamped_y = pos_y.clamp(0.0, max_y as f64);
        (clamped_x, clamped_y).into()
    }
}
