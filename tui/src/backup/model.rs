use anyhow::{Error, Result, anyhow};
use crossterm::event::{Event, KeyCode};
use dbkp_core::{
    DbBkp,
    databases::DatabaseConnection,
    storage::provider::{StorageConfig, StorageProvider},
};

use crate::{
    backup::view::BackupView,
    configs::Configs,
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
    pub configs: Configs,
    pub selection_mode: SelectionMode,
    pub highlight_database_id: String,
    pub highlight_storage_id: String,
    pub selected_database_id: Option<String>,
    pub selected_storage_id: Option<String>,
}

impl BackupModel {
    pub fn new() -> Result<BackupModel> {
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
                    selection_mode: SelectionMode::DB,
                    configs,
                    highlight_database_id: database_config.id.clone(),
                    highlight_storage_id: storage_id,
                    selected_database_id: None,
                    selected_storage_id: None,
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

    pub fn save_selection(&mut self) {
        let database_configs = self.configs.get_database_configs();
        let storage_configs = self.configs.get_storage_configs();

        match self.selection_mode {
            SelectionMode::DB => {
                let selected_config = database_configs
                    .iter()
                    .find(|config| config.id == self.highlight_database_id)
                    .unwrap();

                self.selected_database_id = Some(selected_config.id.clone());
                self.selection_mode = SelectionMode::Storage;
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
        }
    }
}

impl Model for BackupModel {
    fn run_hook(&mut self) -> Result<Option<Box<dyn View>>> {
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

        if database_config.is_some() && storage_config.is_some() {
            let database_config = database_config.unwrap().clone();
            let storage_config = storage_config.unwrap().clone();

            self.selected_database_id = None;
            self.selected_storage_id = None;

            // Create temp tokio runtime and run it in a separate thread to keep the sync flow
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let thread: std::thread::JoinHandle<std::result::Result<(), Error>> =
                std::thread::spawn(move || {
                    runtime.block_on(async move {
                        let database_connection = match DatabaseConnection::new(database_config)
                            .await
                        {
                            Ok(connection) => connection,
                            Err(e) => {
                                return Err(anyhow!("Failed to create database connection: {}", e));
                            }
                        };

                        let storage_provider = match StorageProvider::new(storage_config) {
                            Ok(provider) => provider,
                            Err(e) => {
                                return Err(anyhow!("Failed to create storage provider: {}", e));
                            }
                        };

                        let db_bkp = DbBkp::new(database_connection, storage_provider);
                        match db_bkp.backup().await {
                            Ok(_) => Ok(()),
                            Err(e) => Err(anyhow!("Failed to backup: {}", e)),
                        }
                    })
                });

            self.selected_database_id = None;
            self.selected_storage_id = None;

            match thread.join() {
                Ok(Ok(_)) => {
                    return Ok(Some(Box::new(HomeView::new(HomeModel::new()?))));
                }
                Ok(Err(e)) => {
                    self.selected_database_id = None;
                    self.selected_storage_id = None;
                    return Err(anyhow!("{}", e));
                }
                Err(e) => {
                    self.selected_database_id = None;
                    self.selected_storage_id = None;
                    return Err(anyhow!("Thread panicked during backup operation: {:?}", e));
                }
            }
        }

        Ok(Some(Box::new(BackupView::new(self.clone()))))
    }

    fn handle_event(&mut self, event: &Event) -> Result<Option<Box<dyn View>>> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => return Ok(Some(Box::new(HomeView::new(HomeModel::new()?)))),
                KeyCode::Down => {
                    self.select_next();
                }
                KeyCode::Up => {
                    self.select_previous();
                }
                KeyCode::Enter => {
                    self.save_selection();
                }
                KeyCode::Left => {
                    self.selection_mode = match self.selection_mode {
                        SelectionMode::Storage => SelectionMode::DB,
                        SelectionMode::DB => SelectionMode::Storage,
                    }
                }
                KeyCode::Right => {
                    self.selection_mode = match self.selection_mode {
                        SelectionMode::Storage => SelectionMode::DB,
                        SelectionMode::DB => SelectionMode::Storage,
                    }
                }
                _ => {}
            }
        }

        Ok(Some(Box::new(BackupView::new(self.clone()))))
    }
}
