use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::Event as CrosstermEvent;
use tokio::sync::mpsc;

use crate::tui::{
    error::view::ErrorView,
    event::Event,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    view::View,
};

#[derive(Clone, Debug)]
pub struct ErrorModel {
    pub event_sender: mpsc::UnboundedSender<Event>,
    pub title: String,
    pub message: String,
}

impl ErrorModel {
    pub fn new(
        event_sender: mpsc::UnboundedSender<Event>,
        title: Option<String>,
        message: String,
    ) -> Self {
        Self {
            event_sender,
            title: title.unwrap_or("Error".to_string()),
            message,
        }
    }

    fn exit(&self) -> Result<Option<Box<dyn View>>> {
        Ok(Some(Box::new(HomeView::new(HomeModel::new(
            self.event_sender.clone(),
        )?))))
    }
}

#[async_trait]
impl Model for ErrorModel {
    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<()> {
        if let CrosstermEvent::Key(key) = event {
            let next_view = match key.code {
                _ => self.exit()?,
            };

            let _ = self.event_sender.send(Event::View(next_view));
            return Ok(());
        }

        let _ = self
            .event_sender
            .send(Event::View(Some(Box::new(ErrorView::new(self.clone())))));

        Ok(())
    }
}
