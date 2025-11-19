use anyhow::Result;
use ratatui::{Terminal, prelude::Backend};

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

        let initial_view = HomeView::new(HomeModel::new(events.sender.clone())?);
        let _ = events
            .sender
            .send(Event::View(Some(Box::new(initial_view.clone()))));

        let app = App {
            running: true,
            view: Box::new(initial_view),
            events,
        };

        Ok(app)
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
                        let _ = self
                            .events
                            .sender
                            .send(Event::View(Some(Box::new(error_view))));
                    }
                }
                Event::View(view) => {
                    self.running = match view {
                        Some(view) => {
                            self.view = view;

                            let running = match terminal.draw(|f| {
                                self.view.render(f);
                            }) {
                                Ok(_) => true,
                                Err(_) => false,
                            };

                            running
                        }
                        None => false,
                    };
                }
            };
        }

        Ok(())
    }
}
