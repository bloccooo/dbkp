use anyhow::{Error, Result};
use ratatui::{Terminal, prelude::Backend};

use crate::{
    event::{Event, EventHandler},
    home::{model::HomeModel, view::HomeView},
    model::Model,
    utils::render_error,
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
        let app = App {
            running: true,
            view: Box::new(HomeView::new(HomeModel::new()?)),
            events: EventHandler::new(),
        };

        Ok(app)
    }

    pub async fn render_frame<B: Backend>(
        &mut self,
        mut model: Box<dyn Model>,
        error: Option<Error>,
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
                render_error(error, f);
            }

            if let Some(error) = error {
                render_error(error, f);
            }
        })?;

        Ok(result)
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> color_eyre::Result<()> {
        while self.running {
            let mut model: Box<dyn Model> = self.view.get_model();

            match self.events.next().await? {
                Event::Tick => {
                    // self.running = self.render_frame(model, None, terminal).await?;
                }
                Event::Crossterm(event) => {
                    let error = match model.handle_event(&event).await {
                        Ok(_) => None,
                        Err(e) => Some(e),
                    };

                    self.running = self.render_frame(model, error, terminal).await?;
                }
            };
        }

        Ok(())
    }
}
