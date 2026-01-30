use std::time::Duration;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use dbkp_core::{
    DbBkp, RestoreOptions,
    databases::DatabaseConnection,
    storage::provider::{StorageConfig, StorageProvider},
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
pub enum RestoreStage {
    Selection,
    RestoreConfig,
}

#[derive(Clone, Debug)]
pub enum SelectionMode {
    Storage,
    Backup,
    Database,
}

#[derive(Clone, Debug)]
pub struct RestoreModel {
    pub event_sender: mpsc::UnboundedSender<Event>,
    pub in_progress: bool,
    pub loading_backups: bool,
    pub configs: Configs,
    pub backups: Vec<String>,
    pub restore_stage: RestoreStage,
    pub selection_mode: SelectionMode,
    pub highlight_storage_id: String,
    pub highlight_database_id: String,
    pub highlighted_backup_id: Option<String>,
    pub selected_storage_id: Option<String>,
    pub selected_database_id: Option<String>,
    pub selected_backup_id: Option<String>,
    pub drop_database: bool,
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
                    event_sender,
                    in_progress: false,
                    loading_backups: false,
                    configs,
                    backups: vec![],
                    restore_stage: RestoreStage::Selection,
                    selection_mode: SelectionMode::Storage,
                    highlight_storage_id: storage_id,
                    highlight_database_id: database_config.id.clone(),
                    highlighted_backup_id: None,
                    selected_storage_id: None,
                    selected_database_id: None,
                    selected_backup_id: None,
                    drop_database: false,
                });
            }
        }

        Err(anyhow!("Unable to find existing configs"))
    }

    fn toggle_drop_database(&mut self) -> Result<Option<Box<dyn View>>> {
        self.drop_database = !self.drop_database;
        Ok(self.self_view())
    }

    fn handle_key_down(&mut self) -> Result<Option<Box<dyn View>>> {
        match self.restore_stage {
            RestoreStage::Selection => self.highlight_next(),
            RestoreStage::RestoreConfig => self.toggle_drop_database(),
        }
    }

    fn handle_key_up(&mut self) -> Result<Option<Box<dyn View>>> {
        match self.restore_stage {
            RestoreStage::Selection => self.highlight_previous(),
            RestoreStage::RestoreConfig => self.toggle_drop_database(),
        }
    }

    fn highlight_next(&mut self) -> Result<Option<Box<dyn View>>> {
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

        Ok(self.self_view())
    }

    fn highlight_previous(&mut self) -> Result<Option<Box<dyn View>>> {
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

        Ok(self.self_view())
    }

    fn cycle_through_columns(&mut self, direction: bool) -> Result<Option<Box<dyn View>>> {
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

        Ok(self.self_view())
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
                        let _ = sender.send(Event::View(Some(Box::new(error_view))));
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
                        let _ = sender.send(Event::View(Some(Box::new(error_view))));
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
                let _ = sender.send(Event::View(Some(Box::new(RestoreView::new(restore_model)))));
            });
        };

        self.loading_backups = true;
        let _ = self
            .event_sender
            .send(Event::View(Some(Box::new(RestoreView::new(self.clone())))));

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
            let drop_database = self.drop_database.clone();

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

                match db_bkp
                    .restore(RestoreOptions {
                        name: backup_id,
                        compression_format: None,
                        drop_database_first: Some(drop_database),
                    })
                    .await
                {
                    Ok(_) => {
                        let _ = sender.send(Event::View(Some(Box::new(home_view)))).unwrap();
                    }
                    Err(e) => {
                        let error_view = ErrorView::new(ErrorModel::new(
                            sender.clone(),
                            Some("Restore Failed".to_string()),
                            e.to_string(),
                        ));
                        let _ = sender.send(Event::View(Some(Box::new(error_view))));
                    }
                };
            });
        };

        Ok(())
    }

    fn try_and_restore(&mut self) -> Result<Option<Box<dyn View>>> {
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
            && matches!(self.restore_stage, RestoreStage::RestoreConfig)
        {
            self.restore()?;
            self.in_progress = true;
        }

        Ok(self.self_view())
    }

    fn self_view(&self) -> Option<Box<dyn View>> {
        Some(Box::new(RestoreView::new(self.clone())))
    }

    fn handle_key_esc(&mut self) -> Result<Option<Box<dyn View>>> {
        match self.restore_stage {
            RestoreStage::Selection => self.exit(),
            RestoreStage::RestoreConfig => {
                let mut model = self.clone();
                model.restore_stage = RestoreStage::Selection;
                Ok(Some(Box::new(RestoreView::new(model))))
            }
        }
    }

    fn handle_key_left(&mut self) -> Result<Option<Box<dyn View>>> {
        match self.restore_stage {
            RestoreStage::Selection => self.cycle_through_columns(false),
            RestoreStage::RestoreConfig => {
                let mut model = self.clone();
                model.selected_backup_id = None;
                model.selected_database_id = None;
                model.selected_storage_id = None;
                model.restore_stage = RestoreStage::Selection;
                Ok(Some(Box::new(RestoreView::new(model))))
            }
        }
    }

    fn handle_key_select(&mut self) -> Result<Option<Box<dyn View>>> {
        self.try_and_restore()?;

        match self.restore_stage {
            RestoreStage::Selection => {
                if self.selected_storage_id.is_some()
                    && self.selected_backup_id.is_some()
                    && self.selected_database_id.is_some()
                {
                    let mut model = self.clone();
                    model.restore_stage = RestoreStage::RestoreConfig;
                    return Ok(Some(Box::new(RestoreView::new(model))));
                }

                self.cycle_through_columns(true)
            }
            RestoreStage::RestoreConfig => Ok(self.self_view()),
        }
    }

    fn exit(&mut self) -> Result<Option<Box<dyn View>>> {
        Ok(Some(Box::new(HomeView::new(HomeModel::new(
            self.event_sender.clone(),
        )?))))
    }
}

#[async_trait]
impl Model for RestoreModel {
    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<Option<Box<dyn View>>> {
        if let CrosstermEvent::Key(key) = event {
            let next_view = match key.code {
                KeyCode::Esc => self.handle_key_esc()?,
                KeyCode::Down => self.handle_key_down()?,
                KeyCode::Up => self.handle_key_up()?,
                KeyCode::Left => self.handle_key_left()?,
                KeyCode::Right | KeyCode::Enter => self.handle_key_select()?,
                _ => self.self_view(),
            };

            return Ok(next_view);
        }

        Ok(self.self_view())
    }
}
