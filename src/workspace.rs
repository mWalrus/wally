use std::collections::VecDeque;

use crate::state::ClientState;

#[derive(Debug)]
pub struct Workspace {
    clients: VecDeque<ClientState>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            clients: VecDeque::new(),
        }
    }
}
