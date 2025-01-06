use crate::types::keybind::KeyModifiers;
use std::collections::HashMap;

use lazy_static::lazy_static;
use smithay::input::keyboard::keysyms;

use crate::types::keybind::{Action, Keybind};

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

pub struct Config {
    pub border_thickness: u8,
    pub border_color_focused: u32,
    pub border_color_unfocused: u32,
    pub workspace_count: usize,
    // MAYBE: gap: u8
    pub keybinds: HashMap<Keybind, Action>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            border_thickness: 2,
            border_color_focused: 0x00ff00,
            border_color_unfocused: 0xff0000,
            workspace_count: 9,
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
                    Action::Spawn("bemenu_run".into()),
                ),
            ]),
        }
    }
}
