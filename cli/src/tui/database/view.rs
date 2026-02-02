use ratatui::Frame;

use crate::tui::{
    database::model::{CurrentInput, DatabaseModel},
    model::Model,
    utils::{InputItem, render_input_form},
    view::View,
};

#[derive(Clone, Debug)]
pub struct DatabaseView {
    database_model: DatabaseModel,
}

impl DatabaseView {
    pub fn new(database_model: DatabaseModel) -> Self {
        DatabaseView { database_model }
    }
}

impl View for DatabaseView {
    fn clone_box(&self) -> Box<dyn View> {
        Box::new(self.clone())
    }

    fn get_model(&self) -> Box<dyn Model> {
        Box::new(self.database_model.clone())
    }

    fn render(&self, frame: &mut Frame) {
        let items = vec![
            InputItem {
                label: "Config Name",
                input: &self.database_model.name_input,
                active: matches!(self.database_model.current_input, CurrentInput::Name),
                obfuscate: false,
            },
            InputItem {
                label: "Database Type",
                input: &self.database_model.type_input,
                active: matches!(self.database_model.current_input, CurrentInput::Type),
                obfuscate: false,
            },
            InputItem {
                label: "Database Name",
                input: &self.database_model.database_input,
                active: matches!(self.database_model.current_input, CurrentInput::Database),
                obfuscate: false,
            },
            InputItem {
                label: "Host",
                input: &self.database_model.host_input,
                active: matches!(self.database_model.current_input, CurrentInput::Host),
                obfuscate: false,
            },
            InputItem {
                label: "Port",
                input: &self.database_model.port_input,
                active: matches!(self.database_model.current_input, CurrentInput::Port),
                obfuscate: false,
            },
            InputItem {
                label: "Username",
                input: &self.database_model.username_input,
                active: matches!(self.database_model.current_input, CurrentInput::Username),
                obfuscate: false,
            },
            InputItem {
                label: "Password",
                input: &self.database_model.password_input,
                active: matches!(self.database_model.current_input, CurrentInput::Password),
                obfuscate: true,
            },
        ];

        render_input_form(frame, "Database Configuration", items, frame.area());
    }
}
