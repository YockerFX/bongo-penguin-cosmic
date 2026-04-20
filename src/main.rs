use std::sync::Mutex;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> cosmic::iced::Result {
    init_logging();
    let _ = tracing_log::LogTracer::init();

    tracing::info!("starting bongo-penguin applet {VERSION}");

    cosmic_applet_bongo_penguin::run()
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let stderr_layer = fmt::layer().with_target(true);

    let file_layer = log_file().map(|f| {
        fmt::layer()
            .with_ansi(false)
            .with_target(true)
            .with_writer(Mutex::new(f))
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(stderr_layer)
        .with(file_layer)
        .init();
}

fn log_file() -> Option<std::fs::File> {
    let home = std::env::var_os("HOME")?;
    let dir = std::path::PathBuf::from(home).join(".cache");
    std::fs::create_dir_all(&dir).ok()?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("bongo-penguin.log"))
        .ok()
}
