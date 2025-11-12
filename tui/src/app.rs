use anyhow::Result;

use crate::{
    home::{model::HomeModel, view::HomeView},
    view::View,
};

#[derive(Clone, Debug)]
pub struct App {
    pub view: Box<dyn View>,
}

impl App {
    pub fn new() -> Result<App> {
        let app = App {
            view: Box::new(HomeView::new(HomeModel::new()?)),
        };

        Ok(app)
    }
}
