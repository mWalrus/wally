use bitflags::bitflags;
use smithay::input::keyboard::{Keysym, ModifiersState};
use tracing::info;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct KeyModifiers: u32 {
        const NONE = 0b0000;
        const CTRL = 0b0001;
        const ALT = 0b0010;
        const SHIFT = 0b0100;
        const SUPER = 0b1000;

        const _ = !0;
    }
}

// usch?????????
impl From<&ModifiersState> for KeyModifiers {
    fn from(modifiers_state: &ModifiersState) -> Self {
        let mut modifiers = KeyModifiers::NONE;

        if modifiers_state.ctrl {
            modifiers |= KeyModifiers::CTRL;
        }

        if modifiers_state.alt {
            modifiers |= KeyModifiers::ALT;
        }

        if modifiers_state.shift {
            modifiers |= KeyModifiers::SHIFT;
        }

        if modifiers_state.logo {
            modifiers |= KeyModifiers::SUPER;
        }

        modifiers
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Keybind {
    pub modifiers: KeyModifiers,
    pub key: Keysym,
}

impl Keybind {
    pub fn new(modifiers: impl Into<KeyModifiers>, key: impl Into<Keysym>) -> Self {
        Self {
            modifiers: modifiers.into(),
            key: key.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    NextWorkspace,
    PrevWorkspace,
    Spawn(String),
    MoveWindowToPrevWorkspace, // TODO
    MoveWindowToNextWorkspace, // TODO
    MoveWindowFloating,        // TODO
    ResizeWindowFloating,      // TODO
    MoveWindowBack,            // TODO
    MoveWindowNext,            // TODO
    RemoveWindow,
}
