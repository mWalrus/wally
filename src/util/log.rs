use backtrace::Backtrace;
use chrono::Local;
use std::{fs::File, path::PathBuf};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub fn init<S: AsRef<str>>(level: Option<S>) {
    let home = std::env::var("HOME").expect("$HOME is not set");
    let log_dir = PathBuf::from(home).join(".local/share/wally/");
    let log_file_name = format!("wally_{}.log", Local::now().format("%Y-%m-%d_%H:%M:%S"));
    let log_file_path = log_dir.join(log_file_name);
    let log_link_path = log_dir.join("latest.log");

    std::fs::create_dir_all(&log_dir).unwrap_or_else(|e| {
        panic!(
            "Unable to create log directory '{}': {e}",
            log_dir.to_string_lossy()
        )
    });

    let log_file = File::create(&log_file_path).unwrap_or_else(|e| {
        panic!(
            "Unable to create log file'{}': {e}",
            log_file_path.to_string_lossy()
        )
    });

    if log_link_path.exists() {
        std::fs::remove_file(&log_link_path).unwrap_or_else(|e| {
            panic!(
                "Unable to remove '{}': {e}",
                log_link_path.to_string_lossy()
            )
        });
    }

    std::os::unix::fs::symlink(&log_file_path, &log_link_path).unwrap_or_else(|e| {
        panic!(
            "Unable to symlink '{}': {e}",
            log_link_path.to_string_lossy()
        )
    });

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(log_file)
        .with_filter(filter(level.as_ref()));

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(filter(level));

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .init();

    set_panic_hook();
}

fn filter<S: AsRef<str>>(level: Option<S>) -> EnvFilter {
    match level {
        Some(level) => EnvFilter::builder().parse_lossy(level),
        None => EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    }
}

fn set_panic_hook() {
    std::panic::set_hook(Box::new(move |info| {
        let backtrace = Backtrace::new();

        let thread = std::thread::current();
        let thread = thread.name().unwrap_or("<unnamed>");

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };

        match info.location() {
            Some(location) => {
                tracing::error!(
                    target: "panic",
                    "thread '{}' panicked at '{}': {}:{}{:?}",
                    thread,
                    msg,
                    location.file(),
                    location.line(),
                    backtrace
                );
            }
            None => tracing::error!(
                target: "panic",
                "thread '{}' panicked at '{}'{:?}",
                thread,
                msg,
                backtrace
            ),
        }
    }));
}
