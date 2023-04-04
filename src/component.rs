mod authentication;
mod home;
mod root;

pub use authentication::*;
pub use root::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Consumed,
    NotConsumed,
}
