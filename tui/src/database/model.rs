use anyhow::{Result, anyhow};
use crossterm::event::{Event, KeyCode};
use dbkp_core::databases::{ConnectionType, DatabaseConfig};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    configs::Configs,
    database::view::DatabaseView,
    home::{model::HomeModel, view::HomeView},
    model::Model,
    view::View,
};

#[derive(Clone, Debug)]
pub enum CurrentInput {
    Type,
    Name,
    Database,
    Host,
    Port,
    Username,
    Password,
}

#[derive(Clone, Debug)]
pub struct DatabaseModel {
    pub current_input: CurrentInput,
    pub type_input: Input,
    pub name_input: Input,
    pub database_input: Input,
    pub host_input: Input,
    pub port_input: Input,
    pub username_input: Input,
    pub password_input: Input,
}

impl DatabaseModel {
    pub fn new() -> DatabaseModel {
        DatabaseModel {
            type_input: Input::new("postgresql".to_string()),
            name_input: Input::new("".to_string()),
            database_input: Input::new("".to_string()),
            host_input: Input::new("localhost".to_string()),
            port_input: Input::new("5432".to_string()),
            username_input: Input::new("".to_string()),
            password_input: Input::new("".to_string()),
            current_input: CurrentInput::Name,
        }
    }

    pub fn next_input(&mut self) {
        self.current_input = match self.current_input {
            CurrentInput::Name => CurrentInput::Type,
            CurrentInput::Type => CurrentInput::Database,
            CurrentInput::Database => CurrentInput::Host,
            CurrentInput::Host => CurrentInput::Port,
            CurrentInput::Port => CurrentInput::Username,
            CurrentInput::Username => CurrentInput::Password,
            CurrentInput::Password => CurrentInput::Name,
        };
    }

    pub fn previous_input(&mut self) {
        self.current_input = match self.current_input {
            CurrentInput::Name => CurrentInput::Password,
            CurrentInput::Type => CurrentInput::Name,
            CurrentInput::Database => CurrentInput::Name,
            CurrentInput::Host => CurrentInput::Database,
            CurrentInput::Port => CurrentInput::Host,
            CurrentInput::Username => CurrentInput::Port,
            CurrentInput::Password => CurrentInput::Username,
        };
    }

    fn input_filled(&self) -> bool {
        !self.name_input.value().is_empty()
            && !self.database_input.value().is_empty()
            && !self.host_input.value().is_empty()
            && !self.port_input.value().is_empty()
            && !self.username_input.value().is_empty()
            && !self.password_input.value().is_empty()
    }

    fn validate_inputs(&self) -> Result<()> {
        if self.name_input.value().is_empty() {
            return Err(anyhow!("Name is required"));
        }

        if self.database_input.value().is_empty() {
            return Err(anyhow!("Database is required"));
        }

        if self.host_input.value().is_empty() {
            return Err(anyhow!("Host is required"));
        }

        if self.username_input.value().is_empty() {
            return Err(anyhow!("Username is required"));
        }

        if self.password_input.value().is_empty() {
            return Err(anyhow!("Password is required"));
        }

        match self.port_input.value().parse::<u16>() {
            Ok(_) => {}
            Err(e) => return Err(anyhow!("Invalid port: {}", e)),
        };

        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        let mut config = Configs::load()?;

        let id = cuid2::create_id();

        self.validate_inputs()?;

        let new_database_config = DatabaseConfig {
            id,
            name: self.name_input.value().to_string(),
            connection_type: match self.type_input.value() {
                "postgresql" => ConnectionType::PostgreSql,
                "mysql" => ConnectionType::MySql,
                _ => return Err(anyhow!("Invalid database type")),
            },
            database: self.database_input.value().to_string(),
            host: self.host_input.value().to_string(),
            port: self.port_input.value().parse::<u16>().unwrap(),
            username: self.username_input.value().to_string(),
            password: Some(self.password_input.value().to_string()),
            ssh_tunnel: None,
        };

        config.add_database_config(new_database_config)?;

        Ok(())
    }
}

impl Model for DatabaseModel {
    fn handle_event(&mut self, event: &Event) -> Result<Option<Box<dyn View>>> {
        match self.current_input {
            CurrentInput::Type => {
                self.type_input.handle_event(event);
            }
            CurrentInput::Name => {
                self.name_input.handle_event(event);
            }
            CurrentInput::Database => {
                self.database_input.handle_event(event);
            }
            CurrentInput::Host => {
                self.host_input.handle_event(event);
            }
            CurrentInput::Port => {
                self.port_input.handle_event(event);
            }
            CurrentInput::Username => {
                self.username_input.handle_event(event);
            }
            CurrentInput::Password => {
                self.password_input.handle_event(event);
            }
        };

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Down => {
                    self.next_input();
                }
                KeyCode::Tab => {
                    self.next_input();
                }
                KeyCode::Up => {
                    self.previous_input();
                }
                KeyCode::Enter => {
                    if self.input_filled() {
                        self.save()?;
                        let home_model = HomeModel::new()?;
                        return Ok(Some(Box::new(HomeView::new(home_model))));
                    } else {
                        self.next_input();
                    }
                }
                _ => {}
            }
        }

        Ok(Some(Box::new(DatabaseView::new(self.clone()))))
    }
}
