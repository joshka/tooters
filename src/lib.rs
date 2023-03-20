use std::error;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub mod ui;
