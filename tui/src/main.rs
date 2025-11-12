use anyhow::{Result, anyhow};
use crossterm::event::{self};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Backend,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{app::App, model::Model};
mod app;
mod backup;
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool> {
    loop {
        let event = event::read()?;
        let mut exit = false;
        terminal.draw(|f| {
            let mut model: Box<dyn Model> = app.view.get_model();
            match model.handle_event(&event) {
                Ok(some_view) => {
                    if let Some(view) = some_view {
                        app.view = view;
                    } else {
                        exit = true
                    }
                }
                Err(e) => {
                    let popup_block = Block::default()
                        .title("Error")
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Red));

                    let paragraph = Paragraph::new(e.to_string())
                        .block(popup_block)
                        .wrap(Wrap { trim: true });

                    let area = centered_rect(60, 25, f.area());
                    f.render_widget(paragraph, area);
                }
            }

            app.view.render(f);
        })?;

        if exit {
            break Ok(true);
        }
    }
}
