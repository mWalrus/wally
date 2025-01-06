use smithay::output::Output;

use crate::{config::CONFIG, workspace::Workspace};

#[derive(Debug)]
pub struct Monitor {
    workspaces: Vec<Workspace>,
    output: Output,
}

impl Monitor {
    pub fn new(output: Output) -> Self {
        let workspaces = (0..=CONFIG.workspace_count)
            .map(|_| Workspace::default())
            .collect();
        Self { workspaces, output }
    }

    pub fn output_ref(&self) -> &Output {
        &self.output
    }

    pub fn output_clone(&self) -> Output {
        self.output.clone()
    }
}
