use ratatui::{
    Frame,
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use crate::tui::{model::Model, success::model::SuccessModel, view::View};

#[derive(Clone, Debug)]
pub struct SuccessView {
    pub model: SuccessModel,
}

impl SuccessView {
    pub fn new(model: SuccessModel) -> Self {
        Self { model }
    }
}

impl View for SuccessView {
    fn clone_box(&self) -> Box<dyn View> {
        Box::new(self.clone())
    }

    fn get_model(&self) -> Box<dyn Model> {
        Box::new(self.model.clone())
    }

    fn render(&self, frame: &mut Frame) {
        let block = Block::new()
            .title(" Success ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::all())
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::Green))
            .padding(Padding::uniform(1));

        let paragraph = Paragraph::new(self.model.message.clone())
            .style(Style::default().fg(Color::Green))
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, frame.area());
    }
}
