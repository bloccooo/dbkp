use anyhow::Result;
use crossterm::event::Event as CrosstermEvent;
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

        let app = App {
            running: true,
            view: Box::new(initial_view),
            events,
        };

        Ok(app)
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> color_eyre::Result<()> {
        // Initial render
        terminal.draw(|f| self.view.render(f))?;

        while self.running {
            match self.events.next().await? {
                Event::Tick => {}
                Event::Crossterm(event) => {
                    self.handle_crossterm_event(&event, terminal).await;
                }
                Event::View(next_view) => {
                    self.update_view(next_view, terminal);
                }
            };
        }

        Ok(())
    }

    async fn handle_crossterm_event<B: Backend>(
        &mut self,
        event: &CrosstermEvent,
        terminal: &mut Terminal<B>,
    ) {
        let mut model: Box<dyn Model> = self.view.get_model();

        match model.handle_event(event).await {
            Ok(next_view) => {
                self.update_view(next_view, terminal);
            }
            Err(e) => {
                let error_view = ErrorView::new(ErrorModel::new(
                    self.events.sender.clone(),
                    Some("Event Error".to_string()),
                    e.to_string(),
                ));
                self.update_view(Some(Box::new(error_view)), terminal);
            }
        }
    }

    fn update_view<B: Backend>(&mut self, view: Option<Box<dyn View>>, terminal: &mut Terminal<B>) {
        self.running = match view {
            Some(view) => {
                self.view = view;
                terminal.draw(|f| self.view.render(f)).is_ok()
            }
            None => false,
        };
    }
}
