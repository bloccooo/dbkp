use crate::{
    configs::Configs,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    storage::view::{LocalStorageView, StorageView},
    view::View,
};
use anyhow::{Result, anyhow};
use crossterm::event::{Event, KeyCode};
use dbkp_core::storage::provider::{LocalStorageConfig, StorageConfig};
use tui_input::{Input, backend::crossterm::EventHandler};

#[derive(Clone, Debug)]
pub enum CurrentInput {
    ConfigName,
    LocalLocation,
}

#[derive(Clone, Debug)]
pub struct StorageModel {
    pub current_input: CurrentInput,
    pub storage_type_options: Vec<String>,
    pub selected_option_index: i8,
    pub current_storage_config: Option<StorageConfig>,
    pub local_input_location: Input,
    pub input_config_name: Input,
}

impl StorageModel {
    pub fn new() -> Self {
        StorageModel {
            current_input: CurrentInput::ConfigName,
            storage_type_options: vec!["S3".to_string(), "Local".to_string()],
            selected_option_index: 0,
            current_storage_config: None,
            local_input_location: Input::new("".to_string()),
            input_config_name: Input::new("".to_string()),
        }
    }

    pub fn select_next(&mut self) {
        self.selected_option_index = self.selected_option_index + 1;

        if self.selected_option_index >= self.storage_type_options.len() as i8 {
            self.selected_option_index = 0;
        }
    }

    pub fn select_previous(&mut self) {
        self.selected_option_index = self.selected_option_index - 1;

        if self.selected_option_index < 0 {
            self.selected_option_index = self.storage_type_options.len() as i8 - 1;
        }
    }

    pub fn next_input(&mut self) {
        if let Some(storage_config) = &self.current_storage_config {
            self.current_input = match storage_config {
                StorageConfig::Local(_) => match self.current_input {
                    CurrentInput::ConfigName => CurrentInput::LocalLocation,
                    CurrentInput::LocalLocation => CurrentInput::ConfigName,
                },
                StorageConfig::S3(_) => CurrentInput::ConfigName,
            };
        }
    }

    pub fn previous_input(&mut self) {
        // self.current_input = match self.current_input {
        //     CurrentInput::Name => CurrentInput::Password,
        //     CurrentInput::Type => CurrentInput::Name,
        //     CurrentInput::Database => CurrentInput::Name,
        //     CurrentInput::Host => CurrentInput::Database,
        //     CurrentInput::Port => CurrentInput::Host,
        //     CurrentInput::Username => CurrentInput::Port,
        //     CurrentInput::Password => CurrentInput::Username,
        // };
    }

    fn update_current_config(&mut self) {
        match &mut self.current_storage_config {
            Some(StorageConfig::Local(local_config)) => {
                local_config.location = self.local_input_location.value().to_string();
                local_config.name = self.input_config_name.value().to_string();
            }
            Some(StorageConfig::S3(_)) => {
                // S3 case can be handled here when needed
            }
            None => {}
        }
    }

    pub fn get_target_view(&mut self) -> Result<Box<dyn View>> {
        let option = self
            .storage_type_options
            .get(self.selected_option_index as usize)
            .cloned();

        if let Some(option) = option {
            if option == "S3".to_string() {
                return Err(anyhow!("Not implemented"));
            } else if option == "Local" {
                let mut model = self.clone();
                model.current_storage_config = Some(StorageConfig::Local(LocalStorageConfig {
                    id: "1".to_string(),
                    location: "".to_string(),
                    name: "".to_string(),
                }));

                return Ok(Box::new(LocalStorageView::new(model)));
            }
        }

        let view = Box::new(HomeView::new(HomeModel::new()?));
        Ok(view)
    }

    pub fn save(&mut self) -> Result<()> {
        let mut config = Configs::load()?;

        if let Some(storage_config) = &self.current_storage_config {
            config.add_storage_config(storage_config)?;
        }

        Ok(())
    }
}

impl Model for StorageModel {
    fn handle_event(&mut self, event: &Event) -> Result<Option<Box<dyn View>>> {
        match self.current_input {
            CurrentInput::ConfigName => {
                self.input_config_name.handle_event(event);
            }
            CurrentInput::LocalLocation => {
                self.local_input_location.handle_event(event);
            }
        }

        self.update_current_config();

        let next_view: Box<dyn View> = if let Event::Key(key) = event {
            if let Some(current_config) = &self.current_storage_config {
                match current_config {
                    StorageConfig::Local(_) => match key.code {
                        KeyCode::Down => {
                            self.next_input();
                        }
                        KeyCode::Tab => {
                            self.next_input();
                        }
                        KeyCode::Enter => {
                            self.next_input();
                        }
                        KeyCode::Up => {
                            self.previous_input();
                        }

                        _ => {}
                    },
                    StorageConfig::S3(_) => {}
                }

                Box::new(LocalStorageView::new(self.clone()))
            } else {
                match key.code {
                    KeyCode::Down => {
                        self.select_next();
                    }
                    KeyCode::Up => {
                        self.select_previous();
                    }
                    KeyCode::Enter => {
                        let new_view = self.get_target_view()?;
                        return Ok(Some(new_view));
                    }
                    _ => {}
                }

                Box::new(StorageView::new(self.clone()))
            }
        } else {
            Box::new(StorageView::new(self.clone()))
        };

        Ok(Some(next_view))
    }
}
