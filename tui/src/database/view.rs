use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Paragraph},
};

use crate::database::model::{CurrentInput, DatabaseModel};

pub struct DatabaseView {
    database_model: DatabaseModel,
}

impl DatabaseView {
    pub fn new(database_model: DatabaseModel) -> Self {
        DatabaseView { database_model }
    }

    pub fn render(&self, frame: &mut Frame) {
        let inputs_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let width = inputs_layout[0].width.max(3) - 3;
        let scroll = self.database_model.type_input.visual_scroll(width as usize);
        let is_active = matches!(self.database_model.current_input, CurrentInput::Type);

        let block = Block::new()
            .title("Database Type")
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(if is_active {
                Style::default().fg(Color::LightBlue)
            } else {
                Style::default()
            });

        let database_type_input = Paragraph::new(self.database_model.type_input.value())
            .scroll((0, scroll as u16))
            .block(block);

        frame.render_widget(database_type_input, inputs_layout[0]);
        if is_active {
            let x = self.database_model.type_input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((inputs_layout[0].x + x as u16, inputs_layout[0].y + 1));
        }

        let is_active = matches!(self.database_model.current_input, CurrentInput::Host);
        let block = Block::new()
            .title("Host")
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(if is_active {
                Style::default().fg(Color::LightBlue)
            } else {
                Style::default()
            });

        let database_type_input = Paragraph::new(self.database_model.host_input.value())
            .scroll((0, scroll as u16))
            .block(block);

        frame.render_widget(database_type_input, inputs_layout[1]);
        if is_active {
            let x = self.database_model.host_input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((inputs_layout[1].x + x as u16, inputs_layout[1].y + 1));
        }

        let is_active = matches!(self.database_model.current_input, CurrentInput::Port);
        let block = Block::new()
            .title("Port")
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(if is_active {
                Style::default().fg(Color::LightBlue)
            } else {
                Style::default()
            });

        let database_type_input = Paragraph::new(self.database_model.port_input.value())
            .scroll((0, scroll as u16))
            .block(block);

        frame.render_widget(database_type_input, inputs_layout[2]);
        if is_active {
            let x = self.database_model.port_input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((inputs_layout[2].x + x as u16, inputs_layout[2].y + 1));
        }
    }
}
