use crate::{ssd::Border, types::keybind::KeyModifiers};
use std::collections::HashMap;

use smithay::input::keyboard::keysyms;

use crate::types::keybind::{Action, Keybind};

pub struct Config {
    pub border: Border,
    // MAYBE: gap: u8
    pub keybinds: HashMap<Keybind, Action>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            border: Border::new(1, [1., 0., 0., 1.], [0., 1., 0., 1.]),
            keybinds: HashMap::from([
                (
                    Keybind::new(KeyModifiers::SUPER | KeyModifiers::SHIFT, keysyms::KEY_q),
                    Action::Quit,
                ),
                (
                    Keybind::new(KeyModifiers::SUPER, keysyms::KEY_l),
                    Action::NextWorkspace,
                ),
                (
                    Keybind::new(KeyModifiers::SUPER, keysyms::KEY_h),
                    Action::PrevWorkspace,
                ),
                (
                    Keybind::new(
                        KeyModifiers::SUPER | KeyModifiers::SHIFT,
                        keysyms::KEY_Return,
                    ),
                    Action::Spawn("alacritty".into()),
                ),
                (
                    Keybind::new(KeyModifiers::SUPER, keysyms::KEY_p),
                    Action::Spawn("dmenu_run".into()),
                ),
            ]),
        }
    }
}
