use anyhow::Result;
use ratatui::{prelude::Backend, Terminal};

use crate::tui::{
    error::{model::ErrorModel, view::ErrorView},
    event::{Event, EventHandler},
    home::{model::HomeModel, view::HomeView},
    model::Model,
    view::View,
};

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub view: Box<dyn View>,
    pub events: EventHandler,
}

impl App {
    pub fn new() -> Result<App> {
        let events = EventHandler::new();
        let app = App {
            running: true,
            view: Box::new(HomeView::new(HomeModel::new(events.sender.clone())?)),
            events,
        };

        Ok(app)
    }

    pub fn render_frame<B: Backend>(
        &mut self,
        mut model: Box<dyn Model>,
        terminal: &mut Terminal<B>,
    ) -> color_eyre::Result<bool> {
        let mut result = false;

        terminal.draw(|f| {
            let view_error = match model.get_next_view() {
                Ok(some_view) => match some_view {
                    Some(view) => {
                        self.view = view;
                        self.view.render(f);
                        result = true;
                        None
                    }
                    None => {
                        result = false;
                        None
                    }
                },
                Err(e) => Some(e),
            };

            if let Some(error) = view_error {
                let error_view = ErrorView::new(ErrorModel::new(
                    self.events.sender.clone(),
                    Some("View Error".to_string()),
                    error.to_string(),
                ));
                let _ = self.events.sender.send(Event::View(Box::new(error_view)));
            }
        })?;

        Ok(result)
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> color_eyre::Result<()> {
        while self.running {
            let mut model: Box<dyn Model> = self.view.get_model();
            match self.events.next().await? {
                Event::Tick => {
                    // self.running = self.render_frame(model, terminal)?;
                }
                Event::Crossterm(event) => {
                    let error = match model.handle_event(&event).await {
                        Ok(_) => None,
                        Err(e) => Some(e),
                    };

                    if let Some(error) = error {
                        let error_view = ErrorView::new(ErrorModel::new(
                            self.events.sender.clone(),
                            Some("Event Error".to_string()),
                            error.to_string(),
                        ));
                        let _ = self.events.sender.send(Event::View(Box::new(error_view)));
                    } else {
                        self.running = self.render_frame(model, terminal)?;
                    }
                }
                Event::View(view) => {
                    let model = view.get_model();
                    self.view = view;
                    self.running = self.render_frame(model, terminal)?;
                }
            };
        }

        Ok(())
    }
}
