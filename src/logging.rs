use color_eyre::{eyre::WrapErr, Result};
use std::sync::{Arc, RwLock};
use tracing::{metadata::LevelFilter, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};
use xdg::BaseDirectories;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

/// Sets up logging to a file and a collector for the logs that can be used to display them in the
/// UI.
///
/// Returns a tuple containing the logs and a `WorkerGuard` which ensures that buffered logs are
/// flushed to their output in the case of abrupt terminations of a process.
///
/// # Errors
///
/// Returns an error if the environment filter could not be built or if the XDG base directories
/// could not be retrieved.
pub fn init() -> Result<(LogCollector, WorkerGuard)> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .wrap_err("failed to build env filter")?;

    let log_folder = BaseDirectories::with_prefix("tooters")
        .wrap_err("failed to get XDG base directories")?
        .get_state_home(); // usually this will be ~/.local/state/tooters
    let file_appender = tracing_appender::rolling::hourly(log_folder, "tooters.log");
    let (file_appender, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_timer(tracing_subscriber::fmt::time::uptime());

    let log_collector = LogCollector::default();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(file_layer)
        .with(log_collector.clone());

    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("setting default subscriber failed")?;

    Ok((log_collector, guard))
}

/// Thread-safe log collector
#[derive(Debug, Default, Clone)]
pub struct LogCollector {
    logs: Arc<RwLock<Vec<LogMessage>>>,
}

/// Log message struct that contains the log level, target, and message.
#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: Level,
    pub target: String,
    pub message: String,
    pub fields: Vec<(String, String)>,
}

impl LogCollector {
    /// Returns the last `n` log messages.
    ///
    /// # Panics
    ///
    /// Panics if the lock on the logs is poisoned.
    #[must_use]
    pub fn last_n(&self, n: usize) -> Vec<LogMessage> {
        let logs = self.logs.read().expect("failed to lock logs");
        let start_index = logs.len().saturating_sub(n);
        logs[start_index..].to_vec()
    }

    /// Pushes a log message into the collector.
    ///
    /// # Panics
    ///
    /// Panics if the lock on the logs is poisoned.
    pub fn push(&self, log: LogMessage) {
        let mut logs = self.logs.write().expect("failed to lock logs");
        logs.push(log);
    }
}

impl<S> Layer<S> for LogCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        self.push(LogMessage::from(event));
    }
}

impl LogMessage {
    fn new(level: Level, target: String) -> Self {
        Self {
            level,
            target,
            message: String::new(),
            fields: Vec::new(),
        }
    }

    fn to_line(&self) -> Line {
        Line::from_iter([
            self.level.as_str().fg(level_color(self.level)),
            " ".into(),
            self.target.as_str().dim(),
            ": ".dim(),
            self.message.as_str().into(),
        ])
    }
}

impl From<&Event<'_>> for LogMessage {
    fn from(event: &Event) -> Self {
        let metadata = event.metadata();
        let level = metadata.level().to_owned();
        let target = metadata.target().to_owned();
        let mut message = Self::new(level, target);
        event.record(&mut message);
        message
    }
}

impl Visit for LogMessage {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.fields
                .push((field.name().to_string(), format!("{value:?}")));
        }
    }
}

impl Widget for &LogCollector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_lines = area.height as usize;
        let logs = self.last_n(max_lines);
        let text: Text = logs.iter().map(LogMessage::to_line).collect();
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

const fn level_color(level: Level) -> Color {
    match level {
        Level::ERROR => Color::Red,
        Level::WARN => Color::Yellow,
        Level::INFO => Color::Green,
        Level::DEBUG => Color::Blue,
        Level::TRACE => Color::Cyan,
    }
}
