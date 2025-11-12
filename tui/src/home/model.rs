use anyhow::Result;
use crossterm::event::{Event, KeyCode};

use crate::{
    backup::{model::BackupModel, view::BackupView},
    configs::Configs,
    database::{model::DatabaseModel, view::DatabaseView},
    home::view::HomeView,
    model::Model,
    storage::{model::StorageModel, view::StorageView},
    view::View,
};

#[derive(Clone, Debug)]
pub struct HomeModel {
    pub options: Vec<String>,
    pub selected_option_index: i8,
}

impl HomeModel {
    pub fn new() -> Result<HomeModel> {
        let configs = Configs::load()?;

        let options = if configs.get_database_configs().len() > 0
            && configs.get_storage_configs().len() > 0
        {
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

    pub fn get_target_view(&mut self) -> Result<Box<dyn View>> {
        let option = self
            .options
            .get(self.selected_option_index as usize)
            .cloned();

        if let Some(option) = option {
            if option == "Add DB Connection".to_string() {
                return Ok(Box::new(DatabaseView::new(DatabaseModel::new())));
            } else if option == "Add Storage Provider" {
                return Ok(Box::new(StorageView::new(StorageModel::new())));
            } else if option == "Backup DB" {
                let view = BackupView::new(BackupModel::new()?);
                return Ok(Box::new(view));
            }
        }

        let view = Box::new(HomeView::new(HomeModel::new()?));
        Ok(view)
    }
}

impl Model for HomeModel {
    fn run_hook(&mut self) -> Result<Option<Box<dyn View>>> {
        Ok(None)
    }

    fn handle_event(&mut self, event: &Event) -> Result<Option<Box<dyn View>>> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Down => {
                    self.select_next();
                }
                KeyCode::Up => {
                    self.select_previous();
                }
                KeyCode::Right => {
                    let target_view = self.get_target_view()?;
                    return Ok(Some(target_view));
                }
                KeyCode::Enter => {
                    let target_view = self.get_target_view()?;
                    return Ok(Some(target_view));
                }
                _ => {}
            }
        }

        Ok(Some(Box::new(HomeView::new(self.clone()))))
    }
}
