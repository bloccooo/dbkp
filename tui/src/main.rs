use std::io;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{Terminal, prelude::Backend};

use crate::{
    app::{App, CurrentScreen},
    database::model::DatabaseModel,
};
mod app;
mod database;
mod home;
mod storage;
mod ui;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let _res = run_app(&mut terminal, &mut app);
    terminal.show_cursor()?;
    ratatui::restore();
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;
        let event = event::read()?;
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => return Ok(true),
                _ => {}
            }

            match &app.current_screen {
                CurrentScreen::Home(main_screen) => match key.code {
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    KeyCode::Down => {
                        let mut main_screen = main_screen.clone();
                        main_screen.select_next();
                        app.current_screen = CurrentScreen::Home(main_screen);
                    }
                    KeyCode::Up => {
                        let mut main_screen = main_screen.clone();
                        main_screen.select_previous();
                        app.current_screen = CurrentScreen::Home(main_screen);
                    }
                    KeyCode::Enter => {
                        if main_screen.selected_option_index == 2 {
                            app.current_screen = CurrentScreen::Database(DatabaseModel::new());
                        }
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('q') => {
                        return Ok(false);
                    }
                    _ => {}
                },
                CurrentScreen::Storage => todo!(),
                CurrentScreen::Database(database_screen) => match key.code {
                    KeyCode::Down => {
                        let mut database_screen = database_screen.clone();
                        database_screen.next_input();
                        app.current_screen = CurrentScreen::Database(database_screen);
                    }
                    KeyCode::Up => {
                        let mut database_screen = database_screen.clone();
                        database_screen.previous_input();
                        app.current_screen = CurrentScreen::Database(database_screen);
                    }
                    KeyCode::Enter => {
                        let mut database_screen = database_screen.clone();
                        database_screen.next_input();
                        app.current_screen = CurrentScreen::Database(database_screen);
                    }
                    _ => {
                        let mut database_screen = database_screen.clone();
                        database_screen.handle_event(&event);
                        app.current_screen = CurrentScreen::Database(database_screen);
                    }
                },
            }
        }
    }
}
