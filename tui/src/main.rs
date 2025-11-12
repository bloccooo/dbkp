use anyhow::{Result, anyhow};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{Terminal, prelude::Backend};

use crate::{app::App, model::Model};
mod app;
mod configs;
mod database;
mod home;
mod model;
mod storage;
mod utils;
mod view;

fn main() -> Result<()> {
    color_eyre::install().map_err(|e| anyhow!(e))?;
    let mut terminal = ratatui::init();
    let mut app = App::new().map_err(|e| anyhow!(e))?;
    let _res = run_app(&mut terminal, &mut app);
    terminal.show_cursor()?;
    ratatui::restore();
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool> {
    loop {
        terminal.draw(|f| app.view.render(f))?;

        let event = event::read()?;
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => return Ok(true),
                _ => {}
            }

            let mut model: Box<dyn Model> = app.view.get_model();
            if let Some(new_view) = model.handle_event(&event)? {
                app.view = new_view;
            }
        }
    }
}
