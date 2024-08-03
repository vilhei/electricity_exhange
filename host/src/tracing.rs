use color_eyre::eyre::Result;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};
use tui_logger::{set_default_level, set_log_file, TuiTracingSubscriberLayer};

pub fn initialize_logging() -> Result<()> {
    set_default_level(log::LevelFilter::Trace);

    let time_now = chrono::Local::now().format("%Y_%m_%d__%H_%M_%S");
    let file_path = format!("./logs/{time_now}.log");
    set_log_file(&file_path)?;

    tui_logger::set_level_for_target("log", log::LevelFilter::Info);

    tracing_subscriber::registry()
        .with(TuiTracingSubscriberLayer)
        .with(ErrorLayer::default())
        .init();

    info!(target:"App", "Logging initialized");
    Ok(())
}
