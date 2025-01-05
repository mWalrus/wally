mod handlers;

mod backend;
mod config;
mod element;
mod input;
mod render;
mod ssd;
mod state;
mod types;
mod util;
mod winit;

use clap::{Parser, ValueEnum};
use smithay::reexports::{
    calloop::EventLoop,
    wayland_server::{Display, DisplayHandle},
};

use state::WallyState;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "udev")]
    backend: Backend,
    #[arg(long, name = "LEVEL", default_value = "INFO")]
    log: Option<String>,
    #[arg(long, name = "SPAWN")]
    spawn: Option<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum Backend {
    Winit,
    Udev,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    util::log::init(args.log);

    crate::winit::init()?;

    if let Some(command) = args.spawn {
        std::process::Command::new(command).spawn().ok();
    }

    // event_loop.run(None, &mut data, move |_| {
    //     // wally is running
    // })?;

    Ok(())
}

// let output = state.space.outputs().next().unwrap().clone();
// state.space.map_output(&output, (0, 0));

// let mode = Mode {
//     size,
//     refresh: 60_000,
// };

// output.change_current_state(Some(mode), None, None, None);
// output.set_preferred(mode);
