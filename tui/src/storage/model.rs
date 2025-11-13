use crate::{
    configs::Configs,
    event::Event,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    storage::view::{LocalStorageView, S3StorageView, StorageView},
    view::View,
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use dbkp_core::storage::provider::{LocalStorageConfig, S3StorageConfig, StorageConfig};
use tokio::sync::mpsc;
use tui_input::{Input, backend::crossterm::EventHandler};

#[derive(Clone, Debug)]
pub enum CurrentInput {
    ConfigName,
    LocalLocation,
    S3Location,
    S3Bucket,
    S3Region,
    S3Endpoint,
    S3AccessKey,
    S3SecretKey,
}

#[derive(Clone, Debug)]
pub struct StorageModel {
    pub event_sender: mpsc::UnboundedSender<Event>,
    pub exit: bool,
    pub current_input: CurrentInput,
    pub storage_type_options: Vec<String>,
    pub highlighted_option_index: i8,
    pub current_storage_config: Option<StorageConfig>,
    pub local_input_location: Input,
    pub input_config_name: Input,
    pub s3_input_bucket: Input,
    pub s3_input_region: Input,
    pub s3_input_endpoint: Input,
    pub s3_input_access_key: Input,
    pub s3_input_secret_key: Input,
    pub s3_input_location: Input,
}

impl StorageModel {
    pub fn new(event_sender: mpsc::UnboundedSender<Event>) -> Self {
        StorageModel {
            event_sender,
            exit: false,
            current_input: CurrentInput::ConfigName,
            storage_type_options: vec!["S3".to_string(), "Local".to_string()],
            highlighted_option_index: 0,
            current_storage_config: None,
            local_input_location: Input::new("".to_string()),
            input_config_name: Input::new("".to_string()),
            s3_input_bucket: Input::new("".to_string()),
            s3_input_region: Input::new("".to_string()),
            s3_input_endpoint: Input::new("".to_string()),
            s3_input_access_key: Input::new("".to_string()),
            s3_input_secret_key: Input::new("".to_string()),
            s3_input_location: Input::new("".to_string()),
        }
    }

    pub fn go_next(&mut self) {
        self.highlighted_option_index = self.highlighted_option_index + 1;

        if self.highlighted_option_index >= self.storage_type_options.len() as i8 {
            self.highlighted_option_index = 0;
        }
    }

    pub fn go_previous(&mut self) {
        self.highlighted_option_index = self.highlighted_option_index - 1;

        if self.highlighted_option_index < 0 {
            self.highlighted_option_index = self.storage_type_options.len() as i8 - 1;
        }
    }

    pub fn next_input(&mut self) {
        if let Some(storage_config) = &self.current_storage_config {
            self.current_input = match storage_config {
                StorageConfig::Local(_) => match self.current_input {
                    CurrentInput::ConfigName => CurrentInput::LocalLocation,
                    CurrentInput::LocalLocation => CurrentInput::ConfigName,
                    _ => CurrentInput::ConfigName,
                },
                StorageConfig::S3(_) => match self.current_input {
                    CurrentInput::ConfigName => CurrentInput::S3Location,
                    CurrentInput::S3Location => CurrentInput::S3Bucket,
                    CurrentInput::S3Bucket => CurrentInput::S3Region,
                    CurrentInput::S3Region => CurrentInput::S3Endpoint,
                    CurrentInput::S3Endpoint => CurrentInput::S3AccessKey,
                    CurrentInput::S3AccessKey => CurrentInput::S3SecretKey,
                    CurrentInput::S3SecretKey => CurrentInput::ConfigName,
                    _ => CurrentInput::ConfigName,
                },
            };
        }
    }

    pub fn previous_input(&mut self) {
        if let Some(storage_config) = &self.current_storage_config {
            self.current_input = match storage_config {
                StorageConfig::Local(_) => match self.current_input {
                    CurrentInput::ConfigName => CurrentInput::LocalLocation,
                    CurrentInput::LocalLocation => CurrentInput::ConfigName,
                    _ => CurrentInput::ConfigName,
                },
                StorageConfig::S3(_) => match self.current_input {
                    CurrentInput::ConfigName => CurrentInput::S3SecretKey,
                    CurrentInput::S3Location => CurrentInput::ConfigName,
                    CurrentInput::S3Bucket => CurrentInput::S3Location,
                    CurrentInput::S3Region => CurrentInput::S3Bucket,
                    CurrentInput::S3Endpoint => CurrentInput::S3Region,
                    CurrentInput::S3AccessKey => CurrentInput::S3Endpoint,
                    CurrentInput::S3SecretKey => CurrentInput::S3AccessKey,
                    _ => CurrentInput::ConfigName,
                },
            };
        }
    }

    fn is_config_filled(&self) -> bool {
        if let Some(storage_config) = &self.current_storage_config {
            match storage_config {
                StorageConfig::Local(config) => {
                    let is_empty = config.id.is_empty()
                        || config.name.is_empty()
                        || config.location.is_empty();
                    return !is_empty;
                }
                StorageConfig::S3(config) => {
                    let is_empty = config.id.is_empty()
                        || config.name.is_empty()
                        || config.region.is_empty()
                        || config.endpoint.is_none()
                        || config.bucket.is_empty()
                        || config.access_key.is_empty()
                        || config.secret_key.is_empty();
                    return !is_empty;
                }
            }
        }

        return false;
    }

    fn validate_configs(&self) -> Result<()> {
        if let Some(storage_config) = &self.current_storage_config {
            match storage_config {
                StorageConfig::Local(config) => {
                    if config.id.is_empty() {
                        return Err(anyhow!("ID is required"));
                    }
                    if config.name.is_empty() {
                        return Err(anyhow!("Config name is required"));
                    }
                    if config.location.is_empty() {
                        return Err(anyhow!("Location is required"));
                    }
                }
                StorageConfig::S3(config) => {
                    if config.name.is_empty() {
                        return Err(anyhow!("Config name is required"));
                    }
                    if config.bucket.is_empty() {
                        return Err(anyhow!("Bucket is required"));
                    }
                    if config.region.is_empty() {
                        return Err(anyhow!("Region is required"));
                    }
                    if config.endpoint.is_none() {
                        return Err(anyhow!("Endpoint is required"));
                    }
                    if config.access_key.is_empty() {
                        return Err(anyhow!("Access key is required"));
                    }
                    if config.secret_key.is_empty() {
                        return Err(anyhow!("Secret key is required"));
                    }
                    if config.location.is_empty() {
                        return Err(anyhow!("Location is required"));
                    }
                    if config.id.is_empty() {
                        return Err(anyhow!("ID is required"));
                    }
                }
            }
        } else {
            return Err(anyhow!("Storage config is required"));
        }

        Ok(())
    }

    fn update_current_config(&mut self) {
        match &mut self.current_storage_config {
            Some(StorageConfig::Local(config)) => {
                config.location = self.local_input_location.value().to_string();
                config.name = self.input_config_name.value().to_string();
            }
            Some(StorageConfig::S3(config)) => {
                config.bucket = self.s3_input_bucket.value().to_string();
                config.region = self.s3_input_region.value().to_string();
                config.endpoint = if self.s3_input_endpoint.value().is_empty() {
                    None
                } else {
                    Some(self.s3_input_endpoint.value().to_string())
                };
                config.access_key = self.s3_input_access_key.value().to_string();
                config.secret_key = self.s3_input_secret_key.value().to_string();
                config.location = self.s3_input_location.value().to_string();
                config.name = self.input_config_name.value().to_string();
            }
            None => {}
        }
    }

    pub fn save(&mut self) -> Result<()> {
        let mut config = Configs::load()?;

        if let Some(storage_config) = &self.current_storage_config {
            config.add_storage_config(storage_config)?;
        }

        Ok(())
    }
}

#[async_trait]
impl Model for StorageModel {
    fn get_next_view(&mut self) -> Result<Option<Box<dyn View>>> {
        if self.exit {
            return Ok(Some(Box::new(HomeView::new(HomeModel::new(
                self.event_sender.clone(),
            )?))));
        } else if let Some(config) = &self.current_storage_config {
            match config {
                StorageConfig::Local(_) => {
                    return Ok(Some(Box::new(LocalStorageView::new(self.clone()))));
                }
                StorageConfig::S3(_) => {
                    return Ok(Some(Box::new(S3StorageView::new(self.clone()))));
                }
            }
        }

        return Ok(Some(Box::new(StorageView::new(self.clone()))));
    }

    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<()> {
        match self.current_input {
            CurrentInput::ConfigName => {
                self.input_config_name.handle_event(event);
            }
            CurrentInput::LocalLocation => {
                self.local_input_location.handle_event(event);
            }
            CurrentInput::S3Location => {
                self.s3_input_location.handle_event(event);
            }
            CurrentInput::S3Bucket => {
                self.s3_input_bucket.handle_event(event);
            }
            CurrentInput::S3Region => {
                self.s3_input_region.handle_event(event);
            }
            CurrentInput::S3Endpoint => {
                self.s3_input_endpoint.handle_event(event);
            }
            CurrentInput::S3AccessKey => {
                self.s3_input_access_key.handle_event(event);
            }
            CurrentInput::S3SecretKey => {
                self.s3_input_secret_key.handle_event(event);
            }
        }

        self.update_current_config();

        if let CrosstermEvent::Key(key) = event {
            if let Some(current_config) = &self.current_storage_config {
                match current_config {
                    StorageConfig::Local(_) | StorageConfig::S3(_) => match key.code {
                        KeyCode::Esc | KeyCode::Left => {
                            self.current_storage_config = None;
                        }
                        KeyCode::Enter => {
                            if self.is_config_filled() {
                                self.validate_configs()?;
                                self.save()?;
                                self.current_storage_config = None;
                                self.exit = true;
                            } else {
                                self.next_input();
                            }
                        }
                        KeyCode::Down | KeyCode::Tab => {
                            self.next_input();
                        }
                        KeyCode::Up => {
                            self.previous_input();
                        }

                        _ => {}
                    },
                }
            } else {
                let selected_option = self
                    .storage_type_options
                    .get(self.highlighted_option_index as usize)
                    .cloned();

                match key.code {
                    KeyCode::Esc | KeyCode::Left => {
                        self.exit = true;
                    }
                    KeyCode::Down => {
                        self.go_next();
                    }
                    KeyCode::Up => {
                        self.go_previous();
                    }
                    KeyCode::Enter | KeyCode::Right => {
                        if let Some(option) = selected_option {
                            if option == "Local" {
                                self.current_storage_config =
                                    Some(StorageConfig::Local(LocalStorageConfig {
                                        id: cuid2::create_id(),
                                        location: "".into(),
                                        name: "".into(),
                                    }));
                            } else {
                                let default_endpoint: String =
                                    "https://s3.pub1.infomaniak.cloud".into();
                                let default_access_key: String = "".into();
                                let default_secret_key: String = "".into();
                                let default_bucket: String = "".into();
                                let default_region: String = "us-east-1".into();
                                let default_name: String = "".into();
                                let default_location: String = "/".into();

                                self.current_storage_config =
                                    Some(StorageConfig::S3(S3StorageConfig {
                                        id: cuid2::create_id(),
                                        bucket: default_bucket.clone(),
                                        region: default_region.clone(),
                                        endpoint: Some(default_endpoint.clone()),
                                        access_key: default_access_key.clone(),
                                        secret_key: default_secret_key.clone(),
                                        location: default_location.clone(),
                                        name: default_name.clone(),
                                    }));

                                self.s3_input_endpoint = Input::new(default_endpoint);
                                self.s3_input_access_key = Input::new(default_access_key);
                                self.s3_input_secret_key = Input::new(default_secret_key);
                                self.s3_input_bucket = Input::new(default_bucket);
                                self.s3_input_region = Input::new(default_region);
                                self.input_config_name = Input::new(default_name);
                                self.s3_input_location = Input::new(default_location);
                            }
                        }
                    }

                    _ => {}
                }
            }
        };

        Ok(())
    }
}
