use color_eyre::{
    eyre::{self, WrapErr},
    Result,
};
use logging::{LogCollector, LogMessage};
use parking_lot::Mutex;
use std::{panic, sync::Arc};
use tracing::{error, info, metadata::LevelFilter};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

mod app;
mod authentication;
mod config;
mod event;
mod home;
pub mod logging;
mod root;
mod tui;
mod widgets;

#[tokio::main]
async fn main() -> Result<()> {
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
fn setup_logging() -> Result<(Arc<Mutex<Vec<LogMessage>>>, WorkerGuard)> {
    let log_folder = xdg::BaseDirectories::with_prefix("tooters")
        .wrap_err("failed to get XDG base directories")?
        .get_state_home();
    let file_appender = tracing_appender::rolling::hourly(log_folder, "tooters.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .wrap_err("failed to build env filter")?;
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_timer(tracing_subscriber::fmt::time::uptime());
    let log_collector = LogCollector::default();
    let logs = log_collector.logs();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(file_layer)
        .with(log_collector);

    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("setting default subscriber failed")?;

    Ok((logs, guard))
}

/// This replaces the standard color_eyre panic and error hooks with hooks that
/// restore the terminal before printing the panic or error.
pub fn install_hooks() -> Result<()> {
    let hook_builder = color_eyre::config::HookBuilder::default();
    let (panic_hook, eyre_hook) = hook_builder.into_hooks();

    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        tui::restore().unwrap();
        panic_hook(panic_info);
    }));

    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |error| {
        tui::restore().unwrap();
        eyre_hook(error)
    }))?;

    Ok(())
}
