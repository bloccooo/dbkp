use ratatui::{
    Frame,
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use crate::tui::{error::model::ErrorModel, model::Model, view::View};

#[derive(Clone, Debug)]
pub struct ErrorView {
    pub error_model: ErrorModel,
}

impl ErrorView {
    pub fn new(error_model: ErrorModel) -> Self {
        Self { error_model }
    }
}

impl View for ErrorView {
    fn clone_box(&self) -> Box<dyn View> {
        Box::new(self.clone())
    }

    fn get_model(&self) -> Box<dyn Model> {
        Box::new(self.error_model.clone())
    }

    fn render(&self, frame: &mut Frame) {
        let block = Block::new()
            .title(format!(" {} ", self.error_model.title))
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::all())
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::Red))
            .padding(Padding::uniform(1));

        let paragraph = Paragraph::new(self.error_model.message.clone())
            .style(Style::default().fg(Color::Red))
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, frame.area());
    }
}
