mod backend;
mod config;
mod elements;
mod focus;
mod handlers;
mod input;
mod monitor;
mod state;
mod types;
mod util;
mod workspace;

use clap::{Parser, ValueEnum};

use state::WallyState;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "udev")]
    backend: Backend,
    #[arg(long, name = "LEVEL", default_value = "INFO")]
    log: Option<String>,
    #[arg(long, name = "COMMAND")]
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

    crate::backend::winit::init()?;

    if let Some(command) = args.spawn {
        std::process::Command::new(command).spawn().ok();
    }

    Ok(())
}
