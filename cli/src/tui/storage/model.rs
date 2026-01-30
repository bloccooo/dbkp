use crate::tui::{
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

    pub fn go_next(&mut self) -> Result<Option<Box<dyn View>>> {
        self.highlighted_option_index = self.highlighted_option_index + 1;

        if self.highlighted_option_index >= self.storage_type_options.len() as i8 {
            self.highlighted_option_index = 0;
        }

        Ok(self.self_view())
    }

    pub fn go_previous(&mut self) -> Result<Option<Box<dyn View>>> {
        self.highlighted_option_index = self.highlighted_option_index - 1;

        if self.highlighted_option_index < 0 {
            self.highlighted_option_index = self.storage_type_options.len() as i8 - 1;
        }

        Ok(self.self_view())
    }

    pub fn next_input(&mut self) -> Result<Option<Box<dyn View>>> {
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

        Ok(self.self_view())
    }

    pub fn previous_input(&mut self) -> Result<Option<Box<dyn View>>> {
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

        Ok(self.self_view())
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

    fn self_view(&self) -> Option<Box<dyn View>> {
        let model = self.clone();

        let view: Option<Box<dyn View>> = match model.current_storage_config {
            Some(storage_config) => match storage_config {
                StorageConfig::S3(_) => Some(Box::new(S3StorageView::new(self.clone()))),
                StorageConfig::Local(_) => Some(Box::new(LocalStorageView::new(self.clone()))),
            },
            None => Some(Box::new(StorageView::new(self.clone()))),
        };

        view
    }

    fn exit(&mut self) -> Result<Option<Box<dyn View>>> {
        self.current_storage_config = None;
        Ok(Some(Box::new(HomeView::new(HomeModel::new(
            self.event_sender.clone(),
        )?))))
    }
}

#[async_trait]
impl Model for StorageModel {
    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<Option<Box<dyn View>>> {
        // Get the current input and handle paste events (tui-input doesn't support paste)
        let input = match self.current_input {
            CurrentInput::ConfigName => &mut self.input_config_name,
            CurrentInput::LocalLocation => &mut self.local_input_location,
            CurrentInput::S3Location => &mut self.s3_input_location,
            CurrentInput::S3Bucket => &mut self.s3_input_bucket,
            CurrentInput::S3Region => &mut self.s3_input_region,
            CurrentInput::S3Endpoint => &mut self.s3_input_endpoint,
            CurrentInput::S3AccessKey => &mut self.s3_input_access_key,
            CurrentInput::S3SecretKey => &mut self.s3_input_secret_key,
        };

        input.handle_event(event);

        self.update_current_config();

        if let CrosstermEvent::Key(key) = event {
            let next_view = if let Some(current_config) = &self.current_storage_config {
                match current_config {
                    StorageConfig::Local(_) | StorageConfig::S3(_) => match key.code {
                        KeyCode::Esc | KeyCode::Left => {
                            let mut model = self.clone();
                            model.current_storage_config = None;

                            let view: Option<Box<dyn View>> =
                                Some(Box::new(StorageView::new(model)));

                            view
                        }
                        KeyCode::Enter => {
                            if self.is_config_filled() {
                                self.validate_configs()?;
                                self.save()?;
                                self.current_storage_config = None;
                                self.exit()?
                            } else {
                                self.next_input()?
                            }
                        }
                        KeyCode::Down | KeyCode::Tab => self.next_input()?,
                        KeyCode::Up => self.previous_input()?,
                        _ => self.self_view(),
                    },
                }
            } else {
                let selected_option = self
                    .storage_type_options
                    .get(self.highlighted_option_index as usize)
                    .cloned();

                let next_view = match key.code {
                    KeyCode::Esc | KeyCode::Left => self.exit()?,
                    KeyCode::Down => self.go_next()?,
                    KeyCode::Up => self.go_previous()?,
                    KeyCode::Enter | KeyCode::Right => {
                        let next_view: Option<Box<dyn View>> = if let Some(option) = selected_option
                        {
                            if option == "Local" {
                                self.current_storage_config =
                                    Some(StorageConfig::Local(LocalStorageConfig {
                                        id: cuid2::create_id(),
                                        location: "".into(),
                                        name: "".into(),
                                    }));

                                Some(Box::new(LocalStorageView::new(self.clone())))
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

                                Some(Box::new(S3StorageView::new(self.clone())))
                            }
                        } else {
                            self.exit()?
                        };

                        next_view
                    }

                    _ => self.self_view(),
                };

                next_view
            };

            return Ok(next_view);
        };

        Ok(self.self_view())
    }
}
