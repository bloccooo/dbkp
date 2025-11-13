use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use tokio::sync::mpsc;

use crate::{
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
    pub exit: bool,
    pub options: Vec<String>,
    pub highlighted_option_index: i8,
    pub selected_options_index: Option<i8>,
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
            exit: false,
            options,
            selected_options_index: None,
            highlighted_option_index: 0,
            event_sender,
        };

        Ok(home_model)
    }

    pub fn go_next(&mut self) {
        self.highlighted_option_index = self.highlighted_option_index + 1;

        if self.highlighted_option_index >= self.options.len() as i8 {
            self.highlighted_option_index = 0;
        }
    }

    pub fn go_previous(&mut self) {
        self.highlighted_option_index = self.highlighted_option_index - 1;

        if self.highlighted_option_index < 0 {
            self.highlighted_option_index = self.options.len() as i8 - 1;
        }
    }
}

#[async_trait]
impl Model for HomeModel {
    fn get_next_view(&mut self) -> Result<Option<Box<dyn View>>> {
        if self.exit {
            return Ok(None);
        }

        match self.selected_options_index {
            Some(index) => {
                let option = self.options.get(index as usize).cloned();

                if let Some(option) = option {
                    if option == "Add DB Connection".to_string() {
                        return Ok(Some(Box::new(DatabaseView::new(DatabaseModel::new(
                            self.event_sender.clone(),
                        )?))));
                    } else if option == "Add Storage Provider" {
                        return Ok(Some(Box::new(StorageView::new(StorageModel::new(
                            self.event_sender.clone(),
                        )))));
                    } else if option == "Backup DB" {
                        let view = BackupView::new(BackupModel::new(self.event_sender.clone())?);
                        return Ok(Some(Box::new(view)));
                    } else if option == "Restore DB" {
                        let view = RestoreView::new(RestoreModel::new(self.event_sender.clone())?);
                        return Ok(Some(Box::new(view)));
                    } else if option == "Open Configs Folder" {
                        let _ = std::process::Command::new("open")
                            .arg(Configs::load()?.config_path.parent().unwrap())
                            .spawn();
                        return Ok(None);
                    }
                }
            }
            None => {}
        }

        Ok(Some(Box::new(HomeView::new(self.clone()))))
    }

    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<()> {
        if let CrosstermEvent::Key(key) = event {
            match key.code {
                KeyCode::Esc => {
                    self.exit = true;
                }
                KeyCode::Down => {
                    self.go_next();
                }
                KeyCode::Up => {
                    self.go_previous();
                }
                KeyCode::Right => {
                    let _ = self
                        .selected_options_index
                        .insert(self.highlighted_option_index);
                }
                KeyCode::Enter => {
                    let _ = self
                        .selected_options_index
                        .insert(self.highlighted_option_index);
                }
                _ => {}
            }
        }

        self.event_sender.send(Event::Tick)?;

        Ok(())
    }
}
