use anyhow::{Result, anyhow};
use async_trait::async_trait;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use dbkp_core::databases::{ConnectionType, DatabaseConfig};
use tokio::sync::mpsc;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::tui::{
    configs::Configs,
    database::view::DatabaseView,
    event::Event,
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
    pub event_sender: mpsc::UnboundedSender<Event>,
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
    pub fn new(event_sender: mpsc::UnboundedSender<Event>) -> Result<DatabaseModel> {
        let database_model = DatabaseModel {
            event_sender,
            type_input: Input::new("postgresql".to_string()),
            name_input: Input::new("".to_string()),
            database_input: Input::new("".to_string()),
            host_input: Input::new("localhost".to_string()),
            port_input: Input::new("5432".to_string()),
            username_input: Input::new("".to_string()),
            password_input: Input::new("".to_string()),
            current_input: CurrentInput::Name,
        };

        Ok(database_model)
    }

    pub fn next_input(&mut self) -> Result<Option<Box<dyn View>>> {
        self.current_input = match self.current_input {
            CurrentInput::Name => CurrentInput::Type,
            CurrentInput::Type => CurrentInput::Database,
            CurrentInput::Database => CurrentInput::Host,
            CurrentInput::Host => CurrentInput::Port,
            CurrentInput::Port => CurrentInput::Username,
            CurrentInput::Username => CurrentInput::Password,
            CurrentInput::Password => CurrentInput::Name,
        };

        Ok(self.self_view())
    }

    pub fn previous_input(&mut self) -> Result<Option<Box<dyn View>>> {
        self.current_input = match self.current_input {
            CurrentInput::Name => CurrentInput::Password,
            CurrentInput::Type => CurrentInput::Name,
            CurrentInput::Database => CurrentInput::Name,
            CurrentInput::Host => CurrentInput::Database,
            CurrentInput::Port => CurrentInput::Host,
            CurrentInput::Username => CurrentInput::Port,
            CurrentInput::Password => CurrentInput::Username,
        };

        Ok(self.self_view())
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
        };

        config.add_database_config(new_database_config)?;

        Ok(())
    }

    fn exit(&self) -> Result<Option<Box<dyn View>>> {
        Ok(Some(Box::new(HomeView::new(HomeModel::new(
            self.event_sender.clone(),
        )?))))
    }

    fn self_view(&self) -> Option<Box<dyn View>> {
        Some(Box::new(DatabaseView::new(self.clone())))
    }
}

#[async_trait]
impl Model for DatabaseModel {
    async fn handle_event(&mut self, event: &CrosstermEvent) -> Result<Option<Box<dyn View>>> {
        // Get the current input and handle paste events (tui-input doesn't support paste)
        let input = match self.current_input {
            CurrentInput::Type => &mut self.type_input,
            CurrentInput::Name => &mut self.name_input,
            CurrentInput::Database => &mut self.database_input,
            CurrentInput::Host => &mut self.host_input,
            CurrentInput::Port => &mut self.port_input,
            CurrentInput::Username => &mut self.username_input,
            CurrentInput::Password => &mut self.password_input,
        };

        input.handle_event(event);

        if let CrosstermEvent::Key(key) = event {
            let next_view: Option<Box<dyn View>> = match key.code {
                KeyCode::Esc => self.exit()?,
                KeyCode::Down => self.next_input()?,
                KeyCode::Tab => self.next_input()?,
                KeyCode::Up => self.previous_input()?,
                KeyCode::Enter => {
                    if self.input_filled() {
                        self.save()?;
                        self.exit()?
                    } else {
                        self.next_input()?
                    }
                }
                _ => self.self_view(),
            };

            return Ok(next_view);
        }

        Ok(self.self_view())
    }
}
