use ratatui::{
    Frame, symbols,
    widgets::{Block, Borders},
};

use crate::{
    home::model::HomeModel,
    model::Model,
    utils::{ListItem, create_list},
    view::View,
};

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
                let highlighted = (self.home_model.highlighted_option_index as usize)
                    == self
                        .home_model
                        .options
                        .iter()
                        .position(|x| x == it)
                        .unwrap();

                ListItem {
                    label: it.clone(),
                    highlighted,
                    selected: false,
                }
            })
            .collect();

        let list = create_list(items).block(block);
        frame.render_widget(list, frame.area());
    }
}
