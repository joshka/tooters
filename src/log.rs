use core::fmt;
use std::sync::{Arc, Mutex};
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

pub struct LogCollector {
    logs: Arc<Mutex<Vec<LogMessage>>>,
}

pub struct LogMessage {
    pub level: String,
    pub target: String,
    pub message: String,
}

impl LogCollector {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn logs(&self) -> Arc<Mutex<Vec<LogMessage>>> {
        self.logs.clone()
    }
}

impl<S> Layer<S> for LogCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut logs = self.logs.lock().unwrap();
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
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
}
