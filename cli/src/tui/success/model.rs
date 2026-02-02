use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::Event as CrosstermEvent;
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::tui::{
    event::Event,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    success::view::SuccessView,
    view::View,
};

#[derive(Clone, Debug)]
pub struct SuccessModel {
    pub event_sender: mpsc::UnboundedSender<Event>,
    pub message: String,
    pub timer_handle: Arc<Option<JoinHandle<()>>>,
}

impl SuccessModel {
    pub fn new(event_sender: mpsc::UnboundedSender<Event>, message: String) -> Self {
        let sender = event_sender.clone();
        let timout_sender = event_sender.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;

            if let Ok(home_model) = HomeModel::new(sender) {
                let home_view: Box<dyn View> = Box::new(HomeView::new(home_model));
                let _ = timout_sender.send(Event::View(Some(home_view)));
            }
        });

        Self {
            event_sender,
            message,
            timer_handle: Arc::new(Some(handle)),
        }
    }

    fn exit(&self) -> Result<Option<Box<dyn View>>> {
        Ok(Some(Box::new(HomeView::new(HomeModel::new(
            self.event_sender.clone(),
        )?))))
    }
}

#[async_trait]
impl Model for SuccessModel {
    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<Option<Box<dyn View>>> {
        if let CrosstermEvent::Key(_) = event {
            if let Some(handle) = self.timer_handle.as_ref() {
                handle.abort();
            }

            self.timer_handle = Arc::new(None);

            return self.exit();
        }

        Ok(Some(Box::new(SuccessView::new(self.clone()))))
    }
}
