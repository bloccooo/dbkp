use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use tokio::sync::mpsc;

use crate::tui::{
    backup::{model::BackupModel, view::BackupView},
    configs::Configs,
    database::{model::DatabaseModel, view::DatabaseView},
    event::Event,
    home::view::HomeView,
    model::Model,
    restore::{model::RestoreModel, view::RestoreView},
    storage::{model::StorageModel, view::StorageView},
    view::View,
};

#[derive(Clone, Debug)]
pub struct HomeModel {
    pub options: Vec<String>,
    pub highlighted_option_index: i8,
    pub event_sender: mpsc::UnboundedSender<Event>,
}

impl HomeModel {
    pub fn new(event_sender: mpsc::UnboundedSender<Event>) -> Result<HomeModel> {
        let configs = Configs::load()?;

        let options = if configs.get_database_configs().len() > 0
            && configs.get_storage_configs().len() > 0
        {
            vec![
                "Backup DB".to_string(),
                "Restore DB".to_string(),
                "Add DB Connection".to_string(),
                "Add Storage Provider".to_string(),
                "Open Configs Folder".to_string(),
            ]
        } else {
            vec![
                "Add DB Connection".to_string(),
                "Add Storage Provider".to_string(),
            ]
        };

        let home_model = HomeModel {
            options,
            highlighted_option_index: 0,
            event_sender,
        };

        Ok(home_model)
    }

    fn self_view(&self) -> Option<Box<dyn View>> {
        Some(Box::new(HomeView::new(self.clone())))
    }

    pub fn go_next(&mut self) -> Option<Box<dyn View>> {
        self.highlighted_option_index = self.highlighted_option_index + 1;

        if self.highlighted_option_index >= self.options.len() as i8 {
            self.highlighted_option_index = 0;
        }

        self.self_view()
    }

    pub fn go_previous(&mut self) -> Option<Box<dyn View>> {
        self.highlighted_option_index = self.highlighted_option_index - 1;

        if self.highlighted_option_index < 0 {
            self.highlighted_option_index = self.options.len() as i8 - 1;
        }

        self.self_view()
    }

    pub fn select_option(&mut self) -> Result<Option<Box<dyn View>>> {
        let option = self
            .options
            .get(self.highlighted_option_index as usize)
            .cloned();

        let view: Option<Box<dyn View>> = if let Some(option) = option {
            if option == "Add DB Connection".to_string() {
                Some(Box::new(DatabaseView::new(DatabaseModel::new(
                    self.event_sender.clone(),
                )?)))
            } else if option == "Add Storage Provider" {
                Some(Box::new(StorageView::new(StorageModel::new(
                    self.event_sender.clone(),
                ))))
            } else if option == "Backup DB" {
                Some(Box::new(BackupView::new(BackupModel::new(
                    self.event_sender.clone(),
                )?)))
            } else if option == "Restore DB" {
                Some(Box::new(RestoreView::new(RestoreModel::new(
                    self.event_sender.clone(),
                )?)))
            } else if option == "Open Configs Folder" {
                let _ = std::process::Command::new("open")
                    .arg(Configs::load()?.config_path.parent().unwrap())
                    .spawn();

                None
            } else {
                None
            }
        } else {
            None
        };

        Ok(view)
    }
}

#[async_trait]
impl Model for HomeModel {
    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<()> {
        if let CrosstermEvent::Key(key) = event {
            let next_view: Option<Box<dyn View>> = match key.code {
                KeyCode::Esc => None,
                KeyCode::Down => self.go_next(),
                KeyCode::Up => self.go_previous(),
                KeyCode::Right | KeyCode::Enter => self.select_option()?,
                _ => self.self_view(),
            };

            let _ = self.event_sender.send(Event::View(next_view));
            return Ok(());
        }

        let _ = self.event_sender.send(Event::View(self.self_view()));
        Ok(())
    }
}
