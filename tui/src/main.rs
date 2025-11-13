use anyhow::{Result, anyhow};

use crate::app::App;
mod app;
mod backup;
mod configs;
mod database;
mod error;
mod event;
mod home;
mod model;
mod storage;
mod utils;
mod view;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().map_err(|e| anyhow!(e))?;
    let mut terminal = ratatui::init();
    let mut app = App::new().map_err(|e| anyhow!(e))?;
    let _res = app.run(&mut terminal).await;
    terminal.show_cursor()?;
    ratatui::restore();
    Ok(())
}
