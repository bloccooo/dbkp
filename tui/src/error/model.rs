use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::Event as CrosstermEvent;
use tokio::sync::mpsc;

use crate::{
    error::view::ErrorView,
    event::Event,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    view::View,
};

#[derive(Clone, Debug)]
pub struct ErrorModel {
    pub event_sender: mpsc::UnboundedSender<Event>,
    pub exit: bool,
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
            exit: false,
            title: title.unwrap_or("Error".to_string()),
            message,
        }
    }
}

#[async_trait]
impl Model for ErrorModel {
    fn get_next_view(&mut self) -> Result<Option<Box<dyn View>>> {
        if self.exit {
            let home_model = HomeModel::new(self.event_sender.clone())?;
            return Ok(Some(Box::new(HomeView::new(home_model))));
        }

        Ok(Some(Box::new(ErrorView::new(self.clone()))))
    }

    fn run_hook(&mut self) -> Result<Option<Box<dyn View>>> {
        Ok(None)
    }

    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<()> {
        if let CrosstermEvent::Key(key) = event {
            match key.code {
                _ => {
                    self.exit = true;
                }
            }
        }

        Ok(())
    }
}
