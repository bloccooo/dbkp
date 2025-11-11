use anyhow::Result;

use crate::{database::model::DatabaseModel, home::model::HomeModel};

#[derive(Clone, Debug)]
pub enum CurrentView {
    Home(HomeModel),
    Database(DatabaseModel),
    Exiting,
}

#[derive(Clone, Debug)]
pub struct App {
    pub current_view: CurrentView,
}

impl App {
    pub fn new() -> Result<App> {
        let app = App {
            current_view: CurrentView::Home(HomeModel::new()?),
        };

        Ok(app)
    }
}
