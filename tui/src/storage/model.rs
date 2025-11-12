use crate::{
    configs::Configs,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    storage::view::StorageView,
    view::View,
};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{Event, KeyCode};
use dbkp_core::storage::provider::StorageConfig;
use tui_input::{Input, backend::crossterm::EventHandler};

#[derive(Clone, Debug)]
pub enum CurrentInput {
    ConfigName,
    LocalLocation,
}

#[derive(Clone, Debug)]
pub struct StorageModel {
    pub exit: bool,
    pub current_input: CurrentInput,
    pub storage_type_options: Vec<String>,
    pub highlighted_option_index: i8,
    pub selected_option_index: Option<i8>,
    pub current_storage_config: Option<StorageConfig>,
    pub local_input_location: Input,
    pub input_config_name: Input,
}

impl StorageModel {
    pub fn new() -> Self {
        StorageModel {
            exit: false,
            current_input: CurrentInput::ConfigName,
            storage_type_options: vec!["S3".to_string(), "Local".to_string()],
            highlighted_option_index: 0,
            selected_option_index: None,
            current_storage_config: None,
            local_input_location: Input::new("".to_string()),
            input_config_name: Input::new("".to_string()),
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
    fn run_hook(&mut self) -> Result<Option<Box<dyn View>>> {
        Ok(None)
    }

    fn get_next_view(&mut self) -> Result<Option<Box<dyn View>>> {
        if self.exit {
            return Ok(Some(Box::new(HomeView::new(HomeModel::new()?))));
        }

        return Ok(Some(Box::new(StorageView::new(self.clone()))));
    }

    async fn handle_event(&mut self, event: &Event) -> Result<()> {
        match self.current_input {
            CurrentInput::ConfigName => {
                self.input_config_name.handle_event(event);
            }
            CurrentInput::LocalLocation => {
                self.local_input_location.handle_event(event);
            }
        }

        self.update_current_config();

        if let Event::Key(key) = event {
            if let Some(current_config) = &self.current_storage_config {
                match current_config {
                    StorageConfig::Local(_) => match key.code {
                        KeyCode::Esc => {
                            self.exit = true;
                        }
                        KeyCode::Left => {
                            self.exit = true;
                        }
                        KeyCode::Down => {
                            self.next_input();
                        }
                        KeyCode::Tab => {
                            self.next_input();
                        }
                        KeyCode::Enter => {
                            if matches!(self.current_input, CurrentInput::LocalLocation) {
                                self.save()?;
                                let home_model = HomeModel::new()?;
                                // self.next_view = Some(Box::new(HomeView::new(home_model)));
                            } else {
                                self.next_input();
                            }
                        }
                        KeyCode::Up => {
                            self.previous_input();
                        }

                        _ => {}
                    },
                    StorageConfig::S3(_) => {}
                }
            } else {
                match key.code {
                    KeyCode::Esc => {
                        self.exit = true;
                    }
                    KeyCode::Left => {
                        self.exit = true;
                    }
                    KeyCode::Down => {
                        self.go_next();
                    }
                    KeyCode::Up => {
                        self.go_previous();
                    }
                    KeyCode::Enter => {
                        self.selected_option_index = Some(self.highlighted_option_index);
                    }
                    KeyCode::Right => {
                        self.selected_option_index = Some(self.highlighted_option_index);
                    }
                    _ => {}
                }
            }
        };

        Ok(())
    }
}
