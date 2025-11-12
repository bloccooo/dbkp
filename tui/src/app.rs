use anyhow::Result;
use ratatui::{Terminal, prelude::Backend};

use crate::{
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
        terminal: &mut Terminal<B>,
    ) -> color_eyre::Result<bool> {
        let mut result = false;

        terminal.draw(|f| {
            match model.get_next_view() {
                Ok(some_view) => match some_view {
                    Some(view) => {
                        self.view = view;
                        self.view.render(f);
                        result = true;
                    }
                    None => result = false,
                },
                Err(_) => result = false,
            };
        })?;

        Ok(result)
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> color_eyre::Result<()> {
        while self.running {
            let mut model: Box<dyn Model> = self.view.get_model();

            match self.events.next().await? {
                Event::Tick => {
                    self.running = self.render_frame(model, terminal).await?;
                }
                Event::Crossterm(event) => {
                    let _ = model.handle_event(&event).await;
                    self.running = self.render_frame(model, terminal).await?;
                }
            };
        }

        Ok(())
    }
}
