use std::time::Duration;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use dbkp_core::{
    DbBkp,
    databases::{DatabaseConfig, DatabaseConnection},
    storage::provider::{StorageConfig, StorageProvider},
};
use tokio::sync::mpsc;

use crate::{
    backup::view::BackupView,
    configs::Configs,
    error::{model::ErrorModel, view::ErrorView},
    event::Event,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    view::View,
};

#[derive(Clone, Debug)]
pub enum SelectionMode {
    DB,
    Storage,
}

#[derive(Clone, Debug)]
pub struct BackupModel {
    pub exit: bool,
    pub in_progress: bool,
    pub configs: Configs,
    pub selection_mode: SelectionMode,
    pub highlight_database_id: String,
    pub highlight_storage_id: String,
    pub selected_database_id: Option<String>,
    pub selected_storage_id: Option<String>,
    pub event_sender: mpsc::UnboundedSender<Event>,
}

impl BackupModel {
    pub fn new(event_sender: mpsc::UnboundedSender<Event>) -> Result<BackupModel> {
        let configs = Configs::load()?;
        let database_configs = configs.get_database_configs();
        let storage_configs = configs.get_storage_configs();

        let first_database_config = database_configs.first();
        let first_storage_config = storage_configs.first();

        if let Some(database_config) = first_database_config {
            if let Some(storage_config) = first_storage_config {
                let storage_id = match storage_config {
                    StorageConfig::Local(config) => config.id.clone(),
                    StorageConfig::S3(config) => config.id.clone(),
                };

                return Ok(BackupModel {
                    exit: false,
                    in_progress: false,
                    selection_mode: SelectionMode::DB,
                    configs,
                    highlight_database_id: database_config.id.clone(),
                    highlight_storage_id: storage_id,
                    selected_database_id: None,
                    selected_storage_id: None,
                    event_sender,
                });
            }
        }

        Err(anyhow!("Unable to find existing configs"))
    }

    pub fn select_next(&mut self) {
        let database_configs = self.configs.get_database_configs();
        let storage_configs = self.configs.get_storage_configs();

        match self.selection_mode {
            SelectionMode::DB => {
                let config_index = database_configs
                    .iter()
                    .position(|config| config.id == self.highlight_database_id)
                    .unwrap();

                let next_config_index = (config_index + 1) % database_configs.len();
                self.highlight_database_id = database_configs[next_config_index].id.clone();
            }
            SelectionMode::Storage => {
                let config_index = storage_configs
                    .iter()
                    .position(|config| match config {
                        StorageConfig::Local(config) => config.id == self.highlight_storage_id,
                        StorageConfig::S3(config) => config.id == self.highlight_storage_id,
                    })
                    .unwrap();

                let next_config_index = (config_index + 1) % storage_configs.len();
                self.highlight_storage_id = match &storage_configs[next_config_index] {
                    StorageConfig::Local(config) => config.id.clone(),
                    StorageConfig::S3(config) => config.id.clone(),
                }
            }
        }
    }

    pub fn select_previous(&mut self) {
        let database_configs = self.configs.get_database_configs();
        let storage_configs = self.configs.get_storage_configs();

        match self.selection_mode {
            SelectionMode::DB => {
                let config_index = database_configs
                    .iter()
                    .position(|config| config.id == self.highlight_database_id)
                    .unwrap();

                let previous_config_index = if config_index as i8 - 1 < 0 {
                    database_configs.len() - 1
                } else {
                    config_index - 1
                };

                self.highlight_database_id = database_configs[previous_config_index].id.clone();
            }
            SelectionMode::Storage => {
                let config_index = storage_configs
                    .iter()
                    .position(|config| match config {
                        StorageConfig::Local(config) => config.id == self.highlight_storage_id,
                        StorageConfig::S3(config) => config.id == self.highlight_storage_id,
                    })
                    .unwrap();

                let previous_config_index = if config_index as i8 - 1 < 0 {
                    storage_configs.len() - 1
                } else {
                    config_index - 1
                };

                self.highlight_storage_id = match &storage_configs[previous_config_index] {
                    StorageConfig::Local(config) => config.id.clone(),
                    StorageConfig::S3(config) => config.id.clone(),
                }
            }
        }
    }

    pub fn cycle_through_columns(&mut self, _direction: bool) {
        self.selection_mode = match self.selection_mode {
            SelectionMode::Storage => SelectionMode::DB,
            SelectionMode::DB => SelectionMode::Storage,
        }
    }

    fn try_and_restore(&mut self) -> Result<()> {
        let database_configs = self.configs.get_database_configs();
        let storage_configs = self.configs.get_storage_configs();

        match self.selection_mode {
            SelectionMode::DB => {
                let selected_config = database_configs
                    .iter()
                    .find(|config| config.id == self.highlight_database_id)
                    .unwrap();

                self.selected_database_id = Some(selected_config.id.clone());
            }
            SelectionMode::Storage => {
                let selected_config = storage_configs
                    .iter()
                    .find(|config| match config {
                        StorageConfig::Local(config) => config.id == self.highlight_storage_id,
                        StorageConfig::S3(config) => config.id == self.highlight_storage_id,
                    })
                    .unwrap();

                self.selected_storage_id = match selected_config {
                    StorageConfig::Local(config) => Some(config.id.clone()),
                    StorageConfig::S3(config) => Some(config.id.clone()),
                };
            }
        };

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

        if let Some(database_config) = database_config
            && let Some(storage_config) = storage_config
        {
            self.backup(database_config.clone(), storage_config.clone())?;
        }

        Ok(())
    }

    fn backup(
        &mut self,
        database_config: DatabaseConfig,
        storage_config: StorageConfig,
    ) -> Result<()> {
        self.in_progress = true;
        let sender = self.event_sender.clone();

        let home_view = HomeView::new(HomeModel::new(sender.clone())?);

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

            match db_bkp.backup().await {
                Ok(_) => {
                    let _ = sender.send(Event::View(Box::new(home_view))).unwrap();
                }
                Err(e) => {
                    let error_view = ErrorView::new(ErrorModel::new(
                        sender.clone(),
                        Some("Backup Failed".to_string()),
                        e.to_string(),
                    ));
                    let _ = sender.send(Event::View(Box::new(error_view)));
                }
            };
        });

        return Ok(());
    }
}

#[async_trait]
impl Model for BackupModel {
    fn get_next_view(&mut self) -> Result<Option<Box<dyn View>>> {
        if self.exit {
            return Ok(Some(Box::new(HomeView::new(HomeModel::new(
                self.event_sender.clone(),
            )?))));
        }

        Ok(Some(Box::new(BackupView::new(self.clone()))))
    }

    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<()> {
        if let CrosstermEvent::Key(key) = event {
            match key.code {
                KeyCode::Esc => {
                    self.exit = true;
                }
                KeyCode::Down => {
                    self.select_next();
                }
                KeyCode::Up => {
                    self.select_previous();
                }
                KeyCode::Enter | KeyCode::Right => {
                    self.try_and_restore()?;
                    self.cycle_through_columns(true);
                }
                KeyCode::Left => {
                    self.cycle_through_columns(true);
                }
                _ => {}
            }
        }

        self.event_sender.send(Event::Tick)?;

        Ok(())
    }
}
