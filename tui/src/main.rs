use anyhow::{Result, anyhow};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{Terminal, prelude::Backend};

use crate::{
    app::{App, CurrentView},
    database::model::CurrentInput,
    home::model::HomeModel,
};
mod app;
mod configs;
mod database;
mod home;
mod storage;
mod ui;

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
        terminal.draw(|f| ui::ui(f, app))?;
        let event = event::read()?;
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => return Ok(true),
                _ => {}
            }

            match &app.current_view {
                CurrentView::Home(home_model) => match key.code {
                    KeyCode::Char('q') => {
                        app.current_view = CurrentView::Exiting;
                    }
                    KeyCode::Down => {
                        let mut home_model = home_model.clone();
                        home_model.select_next();
                        app.current_view = CurrentView::Home(home_model);
                    }
                    KeyCode::Up => {
                        let mut home_model = home_model.clone();
                        home_model.select_previous();
                        app.current_view = CurrentView::Home(home_model);
                    }
                    KeyCode::Enter => {
                        let mut home_model = home_model.clone();
                        app.current_view = home_model.get_target_view()?;
                    }
                    _ => {}
                },
                CurrentView::Exiting => match key.code {
                    KeyCode::Char('y') => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('q') => {
                        return Ok(false);
                    }
                    _ => {}
                },
                CurrentView::Database(database_model) => match key.code {
                    KeyCode::Down => {
                        let mut database_model = database_model.clone();
                        database_model.next_input();
                        app.current_view = CurrentView::Database(database_model);
                    }
                    KeyCode::Tab => {
                        let mut database_model = database_model.clone();
                        database_model.next_input();
                        app.current_view = CurrentView::Database(database_model);
                    }
                    KeyCode::Up => {
                        let mut database_model = database_model.clone();
                        database_model.previous_input();
                        app.current_view = CurrentView::Database(database_model);
                    }
                    KeyCode::Enter => {
                        let mut database_model = database_model.clone();

                        match database_model.current_input {
                            CurrentInput::Password => {
                                database_model.save()?;
                                app.current_view = CurrentView::Home(HomeModel::new()?);
                            }
                            _ => {
                                database_model.next_input();
                                app.current_view = CurrentView::Database(database_model);
                            }
                        }
                    }
                    _ => {
                        let mut database_model = database_model.clone();
                        database_model.handle_event(&event);
                        app.current_view = CurrentView::Database(database_model);
                    }
                },
            }
        }
    }
}
