use color_eyre::eyre::Result;
use tracing::{info, instrument, level_filters::LevelFilter};
use tracing_appender::rolling::Rotation;
use tracing_subscriber::{self, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tui_logger::{set_default_level, TuiTracingSubscriberLayer};

// TODO create 2 different logging files? One for general end user and other for debugging with more information?
#[instrument(level = "info")]
pub fn initialize_logging() -> Result<()> {
    tui_logger::set_level_for_target("log", log::LevelFilter::Info);
    set_default_level(log::LevelFilter::Trace);

    let format = fmt::format()
        .with_level(true) // don't include levels in formatted output
        .with_target(true) // don't include targets
        .with_thread_ids(false) // include the thread ID of the current thread
        .with_thread_names(false)
        .with_source_location(true);

    let f = tracing_appender::rolling::Builder::new()
        .filename_suffix("log")
        .rotation(Rotation::HOURLY)
        .build("./logs")
        .unwrap();
    // .with_max_level(tracing::Level::ERROR)
    // .with_min_level(tracing::Level::TRACE);

    let file_layer = fmt::layer()
        .event_format(format)
        .with_writer(f)
        .with_ansi(false)
        .with_filter(LevelFilter::TRACE);
    let tui_layer = TuiTracingSubscriberLayer.with_filter(LevelFilter::TRACE);

    tracing_subscriber::registry()
        .with(tui_layer)
        .with(file_layer)
        .init();

    info!(target:"App", "Logging initialized");
    Ok(())
}
