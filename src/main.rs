use std::panic;
use tooters::app;
use tracing::{error, info, metadata::LevelFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::hourly("./", "tooters.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_writer(non_blocking)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .init();
    panic::set_hook(Box::new(|info| {
        error!("Panic: {:?}", info);
    }));
    app::run().await?;
    info!("Exiting");
    Ok(())
}
