use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::{
    database::model::{CurrentInput, DatabaseModel},
    model::Model,
    utils::render_input,
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
        let inputs_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let width = inputs_layout[0].width.max(3) - 3;
        let scroll = self.database_model.type_input.visual_scroll(width as usize);

        render_input(
            frame,
            &self.database_model.name_input,
            "Config Name",
            matches!(self.database_model.current_input, CurrentInput::Name),
            inputs_layout[0],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.database_model.type_input,
            "Database Type",
            matches!(self.database_model.current_input, CurrentInput::Type),
            inputs_layout[1],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.database_model.database_input,
            "Database Name",
            matches!(self.database_model.current_input, CurrentInput::Database),
            inputs_layout[2],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.database_model.host_input,
            "Host",
            matches!(self.database_model.current_input, CurrentInput::Host),
            inputs_layout[3],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.database_model.port_input,
            "Port",
            matches!(self.database_model.current_input, CurrentInput::Port),
            inputs_layout[4],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.database_model.username_input,
            "Username",
            matches!(self.database_model.current_input, CurrentInput::Username),
            inputs_layout[5],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.database_model.password_input,
            "Password",
            matches!(self.database_model.current_input, CurrentInput::Password),
            inputs_layout[6],
            scroll,
            true,
        );
    }
}
