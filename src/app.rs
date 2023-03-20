use std::time::Duration;

use ratatui::widgets::{Block, Borders};
use tokio::time::sleep;

use crate::{ui::Ui, AppResult};

pub struct App {
    ui: Ui,
    title: String,
}

impl App {
    pub fn new() -> AppResult<Self> {
        let mut ui = Ui::new()?;
        ui.init()?;
        Ok(Self {
            ui,
            title: String::from("Tooters"),
        })
    }

    pub async fn draw(mut self) -> AppResult<()> {
        self.ui.draw(|frame| {
            let size = frame.size();
            let block = Block::default().borders(Borders::ALL).title(self.title);
            frame.render_widget(block, size);
        })?;
        sleep(Duration::from_secs(5)).await;
        Ok(())
    }
}
