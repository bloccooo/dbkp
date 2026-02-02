use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use dbkp_core::{
    DbBkp,
    databases::{DatabaseConfig, DatabaseConnection},
    storage::provider::{StorageConfig, StorageProvider},
};
use tokio::sync::mpsc;

use crate::tui::{
    backup::view::BackupView,
    configs::Configs,
    error::{model::ErrorModel, view::ErrorView},
    event::Event,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    success::{model::SuccessModel, view::SuccessView},
    view::View,
};

#[derive(Clone, Debug)]
pub enum SelectionMode {
    DB,
    Storage,
}

#[derive(Clone, Debug)]
pub enum BackupStage {
    Selection,
    Confirm,
}

#[derive(Clone, Debug)]
pub struct BackupModel {
    pub in_progress: bool,
    pub bytes_written: Arc<AtomicU64>,
    pub configs: Configs,
    pub backup_stage: BackupStage,
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
                    in_progress: false,
                    bytes_written: Arc::new(AtomicU64::new(0)),
                    backup_stage: BackupStage::Selection,
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

    pub fn select_next(&mut self) -> Result<Option<Box<dyn View>>> {
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
        };

        Ok(Some(Box::new(BackupView::new(self.clone()))))
    }

    pub fn select_previous(&mut self) -> Result<Option<Box<dyn View>>> {
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
        };

        Ok(Some(Box::new(BackupView::new(self.clone()))))
    }

    pub fn cycle_through_columns(&mut self, _direction: bool) -> Result<Option<Box<dyn View>>> {
        self.selection_mode = match self.selection_mode {
            SelectionMode::Storage => SelectionMode::DB,
            SelectionMode::DB => SelectionMode::Storage,
        };

        Ok(Some(Box::new(BackupView::new(self.clone()))))
    }

    fn try_and_select(&mut self) -> Result<Option<Box<dyn View>>> {
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

        // If both are selected, transition to Confirm stage
        if self.selected_database_id.is_some() && self.selected_storage_id.is_some() {
            self.backup_stage = BackupStage::Confirm;
            return Ok(Some(Box::new(BackupView::new(self.clone()))));
        }

        Ok(None)
    }

    fn execute_backup(&mut self) -> Result<()> {
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
        self.bytes_written.store(0, std::sync::atomic::Ordering::Relaxed);
        let sender = self.event_sender.clone();
        let bytes_written = self.bytes_written.clone();

        let db_name = database_config.name.clone();
        let storage_name = match &storage_config {
            StorageConfig::Local(config) => config.name.clone(),
            StorageConfig::S3(config) => config.name.clone(),
        };

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
                    let _ = sender.send(Event::View(Some(Box::new(error_view))));
                    return;
                }
                Err(_) => {
                    let error_view = ErrorView::new(ErrorModel::new(
                        sender.clone(),
                        Some("Database Connection Timeout".to_string()),
                        "Timeout".to_string(),
                    ));
                    let _ = sender.send(Event::View(Some(Box::new(error_view))));
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
                    let _ = sender.send(Event::View(Some(Box::new(error_view))));
                    return;
                }
            };

            let db_bkp = DbBkp::new(database_connection, storage_provider);

            match db_bkp.backup_with_progress(None, bytes_written).await {
                Ok(_) => {
                    let success_view = SuccessView::new(SuccessModel::new(
                        sender.clone(),
                        format!(
                            "Backup of \"{}\" database to \"{}\" storage completed successfully.",
                            db_name, storage_name
                        ),
                    ));
                    let _ = sender.send(Event::View(Some(Box::new(success_view))));
                }
                Err(e) => {
                    let error_view = ErrorView::new(ErrorModel::new(
                        sender.clone(),
                        Some("Backup Failed".to_string()),
                        e.to_string(),
                    ));
                    let _ = sender.send(Event::View(Some(Box::new(error_view))));
                }
            };
        });

        return Ok(());
    }
}

#[async_trait]
impl Model for BackupModel {
    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<Option<Box<dyn View>>> {
        if let CrosstermEvent::Key(key) = event {
            let next_view: Option<Box<dyn View>> = match self.backup_stage {
                BackupStage::Selection => match key.code {
                    KeyCode::Esc => Some(Box::new(HomeView::new(HomeModel::new(
                        self.event_sender.clone(),
                    )?))),
                    KeyCode::Down => self.select_next()?,
                    KeyCode::Up => self.select_previous()?,
                    KeyCode::Enter | KeyCode::Right => {
                        if let Some(view) = self.try_and_select()? {
                            Some(view)
                        } else {
                            self.cycle_through_columns(true)?
                        }
                    }
                    KeyCode::Left => self.cycle_through_columns(true)?,
                    _ => Some(Box::new(BackupView::new(self.clone()))),
                },
                BackupStage::Confirm => match key.code {
                    KeyCode::Esc | KeyCode::Left => {
                        // Go back to selection stage
                        self.backup_stage = BackupStage::Selection;
                        self.selected_database_id = None;
                        self.selected_storage_id = None;
                        Some(Box::new(BackupView::new(self.clone())))
                    }
                    KeyCode::Enter => {
                        // Execute backup
                        self.execute_backup()?;
                        Some(Box::new(BackupView::new(self.clone())))
                    }
                    _ => Some(Box::new(BackupView::new(self.clone()))),
                },
            };

            return Ok(next_view);
        }

        Ok(Some(Box::new(BackupView::new(self.clone()))))
    }
}
