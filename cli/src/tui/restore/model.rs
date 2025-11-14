use std::time::Duration;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use dbkp_core::{
    databases::DatabaseConnection,
    storage::provider::{StorageConfig, StorageProvider},
    DbBkp, RestoreOptions,
};
use tokio::sync::mpsc;

use crate::tui::{
    configs::Configs,
    error::{model::ErrorModel, view::ErrorView},
    event::Event,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    restore::view::RestoreView,
    view::View,
};

#[derive(Clone, Debug)]
pub enum SelectionMode {
    Storage,
    Backup,
    Database,
}

#[derive(Clone, Debug)]
pub struct RestoreModel {
    pub exit: bool,
    pub event_sender: mpsc::UnboundedSender<Event>,
    pub in_progress: bool,
    pub loading_backups: bool,
    pub configs: Configs,
    pub backups: Vec<String>,
    pub selection_mode: SelectionMode,
    pub highlight_storage_id: String,
    pub highlight_database_id: String,
    pub highlighted_backup_id: Option<String>,
    pub selected_storage_id: Option<String>,
    pub selected_database_id: Option<String>,
    pub selected_backup_id: Option<String>,
}

impl RestoreModel {
    pub fn new(event_sender: mpsc::UnboundedSender<Event>) -> Result<RestoreModel> {
        let configs = Configs::load()?;

        let storage_configs = configs.get_storage_configs();
        let database_configs = configs.get_database_configs();

        let first_storage_config = storage_configs.first();
        let first_database_config = database_configs.first();

        if let Some(storage_config) = first_storage_config {
            let storage_id = match storage_config {
                StorageConfig::Local(config) => config.id.clone(),
                StorageConfig::S3(config) => config.id.clone(),
            };

            if let Some(database_config) = first_database_config {
                return Ok(RestoreModel {
                    exit: false,
                    event_sender,
                    in_progress: false,
                    loading_backups: false,
                    configs,
                    backups: vec![],
                    selection_mode: SelectionMode::Storage,
                    highlight_storage_id: storage_id,
                    highlight_database_id: database_config.id.clone(),
                    highlighted_backup_id: None,
                    selected_storage_id: None,
                    selected_database_id: None,
                    selected_backup_id: None,
                });
            }
        }

        Err(anyhow!("Unable to find existing configs"))
    }

    fn highlight_next(&mut self) {
        let storage_configs = self.configs.get_storage_configs();
        let database_configs = self.configs.get_database_configs();

        match self.selection_mode {
            SelectionMode::Storage => {
                let current_index = storage_configs
                    .iter()
                    .position(|config| match config {
                        StorageConfig::Local(config) => config.id == self.highlight_storage_id,
                        StorageConfig::S3(config) => config.id == self.highlight_storage_id,
                    })
                    .unwrap_or(0);

                let next_index = (current_index + 1) % storage_configs.len();
                self.highlight_storage_id = match &storage_configs[next_index] {
                    StorageConfig::Local(config) => config.id.clone(),
                    StorageConfig::S3(config) => config.id.clone(),
                };
            }
            SelectionMode::Database => {
                let current_index = database_configs
                    .iter()
                    .position(|config| config.id == self.highlight_database_id)
                    .unwrap_or(0);

                let next_index = (current_index + 1) % database_configs.len();
                self.highlight_database_id = database_configs[next_index].id.clone();
            }
            SelectionMode::Backup => {
                let current_index = self
                    .backups
                    .iter()
                    .position(|backup| Some(backup.clone()) == self.highlighted_backup_id)
                    .unwrap_or(0);

                if self.backups.len() > 0 {
                    let next_index = (current_index + 1) % self.backups.len();
                    if let Some(next_backup) = self.backups.get(next_index) {
                        self.highlighted_backup_id = Some(next_backup.clone())
                    }
                }
            }
        }
    }

    fn highlight_previous(&mut self) {
        let storage_configs = self.configs.get_storage_configs();
        let database_configs = self.configs.get_database_configs();

        match self.selection_mode {
            SelectionMode::Storage => {
                let current_index = storage_configs
                    .iter()
                    .position(|config| match config {
                        StorageConfig::Local(config) => config.id == self.highlight_storage_id,
                        StorageConfig::S3(config) => config.id == self.highlight_storage_id,
                    })
                    .unwrap_or(0);
                let previous_index = if current_index as i8 - 1 < 0 {
                    storage_configs.len() - 1
                } else {
                    current_index - 1
                };
                self.highlight_storage_id = match &storage_configs[previous_index] {
                    StorageConfig::Local(config) => config.id.clone(),
                    StorageConfig::S3(config) => config.id.clone(),
                };
            }
            SelectionMode::Database => {
                let current_index = database_configs
                    .iter()
                    .position(|config| config.id == self.highlight_database_id)
                    .unwrap_or(0);
                let previous_index = if current_index as i8 - 1 < 0 {
                    database_configs.len() - 1
                } else {
                    current_index - 1
                };
                self.highlight_database_id = database_configs[previous_index].id.clone();
            }
            SelectionMode::Backup => {
                let current_index = self
                    .backups
                    .iter()
                    .position(|backup| Some(backup.clone()) == self.highlighted_backup_id)
                    .unwrap_or(0);

                if self.backups.len() > 0 {
                    let previous_index = if current_index as i8 - 1 < 0 {
                        self.backups.len() - 1
                    } else {
                        current_index - 1
                    };

                    if let Some(next_backup) = self.backups.get(previous_index) {
                        self.highlighted_backup_id = Some(next_backup.clone())
                    }
                }
            }
        }
    }

    fn cycle_through_columns(&mut self, direction: bool) {
        match self.selection_mode {
            SelectionMode::Storage => {
                self.selection_mode = if direction {
                    SelectionMode::Backup
                } else {
                    SelectionMode::Database
                };
            }
            SelectionMode::Backup => {
                self.selection_mode = if direction {
                    SelectionMode::Database
                } else {
                    SelectionMode::Storage
                }
            }
            SelectionMode::Database => {
                self.selection_mode = if direction {
                    SelectionMode::Storage
                } else {
                    SelectionMode::Backup
                }
            }
        }
    }

    fn load_backups(&mut self) -> Result<()> {
        let storage_configs = self.configs.get_storage_configs();
        let storage_config = match &self.selected_storage_id {
            Some(id) => storage_configs.iter().find(|config| match config {
                StorageConfig::S3(config) => config.id == *id,
                StorageConfig::Local(config) => config.id == *id,
            }),
            None => None,
        };

        if storage_config.is_some() {
            let sender = self.event_sender.clone();
            let storage_config = storage_config.unwrap().clone();
            let mut restore_model = self.clone();

            tokio::spawn(async move {
                let storage_provider = match StorageProvider::new(storage_config) {
                    Ok(provider) => provider,
                    Err(e) => {
                        let error_view = ErrorView::new(ErrorModel::new(
                            sender.clone(),
                            Some("Storage Provider Error".to_string()),
                            e.to_string(),
                        ));
                        let _ = sender.send(Event::View(Box::new(error_view)));
                        return;
                    }
                };

                let entries = match storage_provider.list().await {
                    Ok(entries) => entries,
                    Err(e) => {
                        let error_view = ErrorView::new(ErrorModel::new(
                            sender.clone(),
                            Some("Fail to load entries".to_string()),
                            e.to_string(),
                        ));
                        let _ = sender.send(Event::View(Box::new(error_view)));
                        return;
                    }
                };

                let backups: Vec<String> = entries
                    .iter()
                    .map(|entry| entry.metadata.name.clone())
                    .collect();

                restore_model.highlighted_backup_id = backups.first().cloned();
                restore_model.backups = backups;
                restore_model.selection_mode = SelectionMode::Backup;
                let _ = sender.send(Event::View(Box::new(RestoreView::new(restore_model))));
            });
        };

        self.loading_backups = true;
        let _ = self
            .event_sender
            .send(Event::View(Box::new(RestoreView::new(self.clone()))));

        Ok(())
    }

    fn restore(&mut self) -> Result<()> {
        let database_configs = self.configs.get_database_configs();
        let storage_configs = self.configs.get_storage_configs();

        let database_config = match &self.selected_database_id {
            Some(id) => database_configs.iter().find(|config| config.id == *id),
            None => None,
        };

        let storage_config = match &self.selected_storage_id {
            Some(id) => storage_configs.iter().find(|config| match config {
                StorageConfig::S3(config) => config.id == *id,
                StorageConfig::Local(config) => config.id == *id,
            }),
            None => None,
        };

        if database_config.is_some()
            && storage_config.is_some()
            && self.selected_backup_id.is_some()
        {
            let sender = self.event_sender.clone();

            let home_view = HomeView::new(HomeModel::new(sender.clone())?);
            let database_config = database_config.unwrap().clone();
            let storage_config = storage_config.unwrap().clone();
            let backup_id = self.selected_backup_id.clone().unwrap();

            tokio::spawn(async move {
                let database_connection_result = tokio::time::timeout(
                    Duration::from_secs(5),
                    DatabaseConnection::new(database_config),
                )
                .await;

                let database_connection = match database_connection_result {
                    Ok(Ok(connection)) => connection,
                    Ok(Err(e)) => {
                        let error_view = ErrorView::new(ErrorModel::new(
                            sender.clone(),
                            Some("Database Connection Error".to_string()),
                            e.to_string(),
                        ));
                        let _ = sender.send(Event::View(Box::new(error_view)));
                        return;
                    }
                    Err(_) => {
                        let error_view = ErrorView::new(ErrorModel::new(
                            sender.clone(),
                            Some("Database Connection Timeout".to_string()),
                            "Timeout".to_string(),
                        ));
                        let _ = sender.send(Event::View(Box::new(error_view)));
                        return;
                    }
                };

                let storage_provider = match StorageProvider::new(storage_config) {
                    Ok(provider) => provider,
                    Err(e) => {
                        let error_view = ErrorView::new(ErrorModel::new(
                            sender.clone(),
                            Some("Storage Provider Error".to_string()),
                            e.to_string(),
                        ));
                        let _ = sender.send(Event::View(Box::new(error_view)));
                        return;
                    }
                };

                let db_bkp = DbBkp::new(database_connection, storage_provider);

                match db_bkp
                    .restore(RestoreOptions {
                        name: backup_id,
                        compression_format: None,
                        drop_database_first: Some(true),
                    })
                    .await
                {
                    Ok(_) => {
                        let _ = sender.send(Event::View(Box::new(home_view))).unwrap();
                    }
                    Err(e) => {
                        let error_view = ErrorView::new(ErrorModel::new(
                            sender.clone(),
                            Some("Restore Failed".to_string()),
                            e.to_string(),
                        ));
                        let _ = sender.send(Event::View(Box::new(error_view)));
                    }
                };
            });
        };

        Ok(())
    }

    fn try_and_restore(&mut self) -> Result<()> {
        match self.selection_mode {
            SelectionMode::Storage => {
                self.selected_storage_id = Some(self.highlight_storage_id.clone());
                self.load_backups()?;
            }
            SelectionMode::Database => {
                self.selected_database_id = Some(self.highlight_database_id.clone());
            }
            SelectionMode::Backup => {
                self.selected_backup_id = self.highlighted_backup_id.clone();
            }
        }

        if self.selected_storage_id.is_some()
            && self.selected_backup_id.is_some()
            && self.selected_database_id.is_some()
        {
            self.restore()?;
            let mut model = self.clone();
            model.in_progress = true;
            let _ = self
                .event_sender
                .send(Event::View(Box::new(RestoreView::new(model))));
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

#[async_trait]
impl Model for RestoreModel {
    fn get_next_view(&mut self) -> Result<Option<Box<dyn View>>> {
        if self.exit {
            return Ok(Some(Box::new(HomeView::new(HomeModel::new(
                self.event_sender.clone(),
            )?))));
        }

        Ok(Some(Box::new(RestoreView::new(self.clone()))))
    }

    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<()> {
        if let CrosstermEvent::Key(key) = event {
            match key.code {
                KeyCode::Esc => self.exit(),
                KeyCode::Down => self.highlight_next(),
                KeyCode::Up => self.highlight_previous(),
                KeyCode::Left => self.cycle_through_columns(false),
                KeyCode::Right => {
                    self.try_and_restore()?;
                    self.cycle_through_columns(true)
                }
                KeyCode::Enter => self.try_and_restore()?,
                _ => {}
            }
        }
        Ok(())
    }
}
