use std::error;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub mod app;
pub mod ui;
pub mod initial_view;
pub mod logged_in_view;

#[cfg(test)]
extern crate tempfile;
