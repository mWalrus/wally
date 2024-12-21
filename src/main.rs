mod handlers;

mod config;
mod grabs;
mod input;
mod state;
mod types;
mod util;
mod winit;

use clap::{Parser, ValueEnum};
use smithay::reexports::{
    calloop::EventLoop,
    wayland_server::{Display, DisplayHandle},
};

use state::Wally;

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

pub struct CalloopData {
    state: Wally,
    display_handle: DisplayHandle,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    util::log::init(args.log);

    let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new()?;

    let display: Display<Wally> = Display::new()?;
    let display_handle = display.handle();
    let state = Wally::new(&mut event_loop, display);

    let mut data = CalloopData {
        state,
        display_handle,
    };

    crate::winit::init(&mut event_loop, &mut data)?;

    if let Some(command) = args.spawn {
        std::process::Command::new(command).spawn().ok();
    }

    event_loop.run(None, &mut data, move |_| {
        // wally is running
    })?;

    Ok(())
}
