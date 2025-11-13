use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{error::model::ErrorModel, model::Model, utils::centered_rect, view::View};

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
        let popup_block = Block::default()
            .title(self.error_model.title.to_string())
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Red));

        let paragraph = Paragraph::new(self.error_model.message.to_string())
            .block(popup_block)
            .wrap(Wrap { trim: true });

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(paragraph, area);
    }
}
