use anyhow::{Error, Result, anyhow};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
mod app;
mod backup;
mod configs;
mod database;
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

fn render_error(error: Error, frame: &mut Frame) {
    let popup_block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Red));

    let paragraph = Paragraph::new(error.to_string())
        .block(popup_block)
        .wrap(Wrap { trim: true });

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(paragraph, area);
}

// async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool> {
//     loop {
//         let event = event::read()?;
//         let mut exit = false;
//         terminal.draw(|f| {
//             let mut model: Box<dyn Model> = app.view.get_model();

//             match model.handle_event(&event) {
//                 Ok(some_view) => {
//                     if let Some(view) = some_view {
//                         app.view = view;
//                         app.view.render(f)
//                     } else {
//                         exit = true
//                     }
//                 }
//                 Err(e) => {
//                     app.view.render(f);
//                     render_error(e, f);
//                 }
//             }
//         })?;

//         terminal.draw(|f| {
//             let mut model: Box<dyn Model> = app.view.get_model();

//             // Hooks are useful to run logic just after the rendering happens
//             match model.run_hook() {
//                 Ok(some_view) => match some_view {
//                     Some(view) => {
//                         println!("{:?}", "oooooo");
//                         app.view = view;
//                         app.view.render(f)
//                     }
//                     None => app.view.render(f),
//                 },
//                 Err(e) => {
//                     app.view.render(f);
//                     render_error(e, f);
//                 }
//             };
//         })?;

//         if exit {
//             break Ok(true);
//         }
//     }
// }
