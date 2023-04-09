use anyhow::Context;
use std::panic;
use tooters::{app, logging::LogCollector};
use tracing::{error, info, metadata::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::hourly("./", "tooters.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

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

    panic::set_hook(Box::new(|info| {
        error!("Panic: {:?}", info);
    }));
    app::run(logs.clone()).await?;
    info!("Exiting");
    Ok(())
}
