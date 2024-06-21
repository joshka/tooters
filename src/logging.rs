use color_eyre::{eyre::WrapErr, Result};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::metadata::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

/// Sets up logging to a file and a collector for the logs that can be used to
/// display them in the UI.
///
/// Returns a tuple containing the logs and a WorkerGuard which ensures that
/// buffered logs are flushed to their output in the case of abrupt terminations
/// of a process.
pub fn init() -> Result<(Arc<Mutex<Vec<LogMessage>>>, WorkerGuard)> {
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

#[derive(Default)]
pub struct LogCollector {
    logs: Arc<Mutex<Vec<LogMessage>>>,
}

pub struct LogMessage {
    pub level: String,
    pub target: String,
    pub message: String,
}

impl LogCollector {
    #[must_use]
    pub fn logs(&self) -> Arc<Mutex<Vec<LogMessage>>> {
        self.logs.clone()
    }
}

impl<S> Layer<S> for LogCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut logs = self.logs.lock();
        let metadata = event.metadata();
        let level = metadata.level().to_string();
        let target = metadata.target();
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let message = visitor.message;
        let log = LogMessage {
            level,
            target: target.to_string(),
            message,
        };
        logs.push(log);
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}

pub struct LogWidget {
    logs: Arc<Mutex<Vec<LogMessage>>>,
}

impl LogWidget {
    pub fn new(logs: Arc<Mutex<Vec<LogMessage>>>) -> Self {
        Self { logs }
    }
}

impl Widget for LogWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let logs = self.logs.lock();
        let max_lines = area.height as usize;
        let start_index = if logs.len() > max_lines {
            logs.len() - max_lines
        } else {
            0
        };

        let text = logs[start_index..]
            .iter()
            .map(|log| {
                let level_color = match log.level.as_str() {
                    "ERROR" => Color::Red,
                    "WARN" => Color::Yellow,
                    "INFO" => Color::Green,
                    "DEBUG" => Color::Blue,
                    "TRACE" => Color::Cyan,
                    _ => Color::White,
                };
                Line::from(vec![
                    Span::styled(&log.level, Style::default().fg(level_color)),
                    Span::raw(" "),
                    Span::styled(&log.target, Style::default().add_modifier(Modifier::DIM)),
                    Span::styled(": ", Style::default().add_modifier(Modifier::DIM)),
                    Span::styled(&log.message, Style::default()),
                ])
            })
            .collect::<Vec<_>>();

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}
