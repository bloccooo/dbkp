use crate::{database::model::DatabaseModel, home::model::HomeModel};

#[derive(Clone, Debug)]
pub enum CurrentScreen {
    Home(HomeModel),
    Database(DatabaseModel),
    Storage,
    Exiting,
}

#[derive(Clone, Debug)]
pub struct App {
    pub current_screen: CurrentScreen,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Home(HomeModel::new()),
        }
    }
}
