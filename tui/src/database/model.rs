use crossterm::event::Event;
use tui_input::{Input, backend::crossterm::EventHandler};

#[derive(Clone, Debug)]
pub enum CurrentInput {
    Type,
    Host,
    Port,
}

#[derive(Clone, Debug)]
pub struct DatabaseModel {
    pub current_input: CurrentInput,
    pub type_input: Input,
    pub host_input: Input,
    pub port_input: Input,
}

impl DatabaseModel {
    pub fn new() -> DatabaseModel {
        DatabaseModel {
            type_input: Input::new("".to_string()),
            host_input: Input::new("".to_string()),
            port_input: Input::new("".to_string()),
            current_input: CurrentInput::Type,
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match self.current_input {
            CurrentInput::Type => {
                self.type_input.handle_event(event);
            }
            CurrentInput::Host => {
                self.host_input.handle_event(event);
            }
            CurrentInput::Port => {
                self.port_input.handle_event(event);
            }
        }
    }

    pub fn next_input(&mut self) {
        self.current_input = match self.current_input {
            CurrentInput::Type => CurrentInput::Host,
            CurrentInput::Host => CurrentInput::Port,
            CurrentInput::Port => CurrentInput::Type,
        };
    }

    pub fn previous_input(&mut self) {
        self.current_input = match self.current_input {
            CurrentInput::Type => CurrentInput::Port,
            CurrentInput::Host => CurrentInput::Type,
            CurrentInput::Port => CurrentInput::Host,
        };
    }
}
