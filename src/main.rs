use std::time::Duration;

use ratatui::{
    backend::Backend,
    widgets::{Block, Borders},
    Frame,
};
use tooters::{ui::Ui, AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    let mut ui = Ui::new()?;
    ui.init()?;
    ui.draw(hello_world)?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}

fn hello_world<B: Backend>(f: &mut Frame<'_, B>) {
    let size = f.size();
    let block = Block::default().borders(Borders::ALL).title("Hello World");
    f.render_widget(block, size);
}
