use anyhow::Result;

use crate::{app::CurrentView, configs::Configs, database::model::DatabaseModel};

#[derive(Clone, Debug)]
pub struct HomeModel {
    pub options: Vec<String>,
    pub selected_option_index: i8,
}

impl HomeModel {
    pub fn new() -> Result<HomeModel> {
        let configs = Configs::load()?;

        let options = if configs.get_database_configs().len() > 0 {
            vec![
                "Backup DB".to_string(),
                "Restore DB".to_string(),
                "Add DB Connection".to_string(),
                "Add Storage Provider".to_string(),
            ]
        } else {
            vec![
                "Add DB Connection".to_string(),
                "Add Storage Provider".to_string(),
            ]
        };

        let home_model = HomeModel {
            options,
            selected_option_index: 0,
        };

        Ok(home_model)
    }

    pub fn select_next(&mut self) {
        self.selected_option_index = self.selected_option_index + 1;

        if self.selected_option_index >= self.options.len() as i8 {
            self.selected_option_index = 0;
        }
    }

    pub fn select_previous(&mut self) {
        self.selected_option_index = self.selected_option_index - 1;

        if self.selected_option_index < 0 {
            self.selected_option_index = self.options.len() as i8 - 1;
        }
    }

    pub fn get_target_view(&mut self) -> Result<CurrentView> {
        let option = self
            .options
            .get(self.selected_option_index as usize)
            .cloned();

        if let Some(option) = option {
            if option == "Add DB Connection".to_string() {
                return Ok(CurrentView::Database(DatabaseModel::new()));
            }
        }

        let view = CurrentView::Home(HomeModel::new()?);
        Ok(view)
    }
}
