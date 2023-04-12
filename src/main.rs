use anyhow::Context;
use parking_lot::Mutex;
use std::{panic, sync::Arc};
use toot_rs::{
    app,
    logging::{LogCollector, LogMessage},
};
use tracing::{error, info, metadata::LevelFilter};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

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
    let log_folder = xdg::BaseDirectories::with_prefix("toot-rs")
        .context("failed to get XDG base directories")?
        .get_state_home();
    let file_appender = tracing_appender::rolling::hourly(log_folder, "toot-rs.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let log_collector = LogCollector::default();
    let logs = log_collector.logs();
    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_timer(tracing_subscriber::fmt::time::uptime()),
        )
        .with(log_collector);

    tracing::subscriber::set_global_default(subscriber)
        .context("setting default subscriber failed")?;

    Ok((logs, guard))
}
