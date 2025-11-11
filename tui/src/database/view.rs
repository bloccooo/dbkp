use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Paragraph},
};
use tui_input::Input;

use crate::database::model::{CurrentInput, DatabaseModel};

pub struct DatabaseView {
    database_model: DatabaseModel,
}

impl DatabaseView {
    pub fn new(database_model: DatabaseModel) -> Self {
        DatabaseView { database_model }
    }

    fn render_input(
        &self,
        frame: &mut Frame,
        input: &Input,
        title: &str,
        is_active: bool,
        area: Rect,
        scroll: usize,
        obfuscate: bool,
    ) {
        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(if is_active {
                Style::default().fg(Color::LightBlue)
            } else {
                Style::default()
            });

        let value = input.value();
        let display_value = if obfuscate {
            "•".repeat(value.len())
        } else {
            value.to_string()
        };

        let paragraph = Paragraph::new(display_value)
            .scroll((0, scroll as u16))
            .block(block);

        frame.render_widget(paragraph, area);

        if is_active {
            let x = input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1));
        }
    }

    pub fn render(&self, frame: &mut Frame) {
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

        self.render_input(
            frame,
            &self.database_model.name_input,
            "Config Name",
            matches!(self.database_model.current_input, CurrentInput::Name),
            inputs_layout[0],
            scroll,
            false,
        );

        self.render_input(
            frame,
            &self.database_model.type_input,
            "Database Type",
            matches!(self.database_model.current_input, CurrentInput::Type),
            inputs_layout[1],
            scroll,
            false,
        );

        self.render_input(
            frame,
            &self.database_model.database_input,
            "Database Name",
            matches!(self.database_model.current_input, CurrentInput::Database),
            inputs_layout[2],
            scroll,
            false,
        );

        self.render_input(
            frame,
            &self.database_model.host_input,
            "Host",
            matches!(self.database_model.current_input, CurrentInput::Host),
            inputs_layout[3],
            scroll,
            false,
        );

        self.render_input(
            frame,
            &self.database_model.port_input,
            "Port",
            matches!(self.database_model.current_input, CurrentInput::Port),
            inputs_layout[4],
            scroll,
            false,
        );

        self.render_input(
            frame,
            &self.database_model.username_input,
            "Username",
            matches!(self.database_model.current_input, CurrentInput::Username),
            inputs_layout[5],
            scroll,
            false,
        );

        self.render_input(
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
