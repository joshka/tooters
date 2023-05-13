use anyhow::Context;
use parking_lot::Mutex;
use std::{panic, sync::Arc};
use toot_rs::{
    app,
    logging::{LogCollector, LogMessage},
};
use tracing::{error, info, metadata::LevelFilter};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter, Registry,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // _guard is a WorkerGuard which ensures that buffered logs are flushed to
    // their output in the case of abrupt terminations of a process.
    let (logs, _guard) = setup_logging()?;
    info!("Starting");
    panic::set_hook(Box::new(|info| {
        error!("Panic: {:?}", info);
    }));
    app::run(logs).await?;
    info!("Exiting");
    Ok(())
}

/// Sets up logging to a file and a collector for the logs that can be used to
/// display them in the UI.
fn setup_logging() -> anyhow::Result<(Arc<Mutex<Vec<LogMessage>>>, WorkerGuard)> {
    // handle logs from the log crate by forwarding them to tracing
    LogTracer::init()?;

    // handle logs from the tracing crate
    let log_folder = xdg::BaseDirectories::with_prefix("toot-rs")
        .context("failed to get XDG base directories")?
        .get_state_home();
    let file_appender = tracing_appender::rolling::hourly(log_folder, "toot-rs.json");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let filter = EnvFilter::default()
        .add_directive("hyper=info".parse()?)
        .add_directive("html5ever=info".parse()?)
        .add_directive("reqwest=info".parse()?)
        .add_directive("mastodon_async=trace".parse()?)
        .add_directive("debug".parse()?);
    let file_layer = fmt::layer()
        .json()
        .with_span_events(FmtSpan::ACTIVE)
        .with_writer(non_blocking)
        .with_filter(filter);

    // collect logs in a collector that can be used to display them in the UI
    let log_collector = LogCollector::default();
    let logs = log_collector.logs();

    let subscriber = Registry::default()
        .with(file_layer)
        .with(log_collector.with_filter(LevelFilter::INFO));

    tracing::subscriber::set_global_default(subscriber)
        .context("setting default subscriber failed")?;

    Ok((logs, guard))
}
