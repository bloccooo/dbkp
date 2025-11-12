use ratatui::{
    Frame,
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem},
};

use crate::{home::model::HomeModel, model::Model, view::View};

#[derive(Clone, Debug)]
pub struct HomeView {
    home_model: HomeModel,
}

impl HomeView {
    pub fn new(home_model: HomeModel) -> Self {
        HomeView { home_model }
    }
}

impl View for HomeView {
    fn clone_box(&self) -> Box<dyn View> {
        Box::new(self.clone())
    }

    fn get_model(&self) -> Box<dyn Model> {
        Box::new(self.home_model.clone())
    }

    fn render(&self, frame: &mut Frame) {
        let block = Block::new()
            .title("DBKP - DB Backup & Restore Tool")
            .borders(Borders::all())
            .border_set(symbols::border::ROUNDED);

        let items: Vec<ListItem> = self
            .home_model
            .options
            .iter()
            .map(|it| {
                if (self.home_model.selected_option_index as usize)
                    == self
                        .home_model
                        .options
                        .iter()
                        .position(|x| x == it)
                        .unwrap()
                {
                    ListItem::from(it.as_str()).style(Style::default().bg(Color::LightBlue))
                } else {
                    ListItem::from(it.as_str())
                }
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_widget(list, frame.area());
    }
}
