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
