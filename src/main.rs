use std::panic;
use tooters::app;
use tracing::{error, info, metadata::LevelFilter};
// use tracing_log::LogTracer;
// use log;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // femme::with_level(log::LevelFilter::Trace);

    // LogTracer::builder()
    //     .ignore_crate("foo") // suppose the `foo` crate is using `tracing`'s log feature
    //     .with_max_level(log::LevelFilter::Trace)
    //     .init()?;

    let file_appender = tracing_appender::rolling::hourly("./", "tooters.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        // .pretty()
        // .full()
        // .compact()
        // .with_max_level(LevelFilter::INFO)
        .with_max_level(LevelFilter::DEBUG)
        // .with_max_level(LevelFilter::TRACE)
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
