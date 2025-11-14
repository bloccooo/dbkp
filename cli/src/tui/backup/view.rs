use dbkp_core::storage::provider::StorageConfig;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    symbols,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tui::{
    backup::model::{BackupModel, SelectionMode},
    model::Model,
    utils::{ListItem, create_list},
    view::View,
};

#[derive(Clone, Debug)]
pub struct BackupView {
    backup_model: BackupModel,
}

impl BackupView {
    pub fn new(backup_model: BackupModel) -> Self {
        Self { backup_model }
    }
}

impl View for BackupView {
    fn clone_box(&self) -> Box<dyn View> {
        Box::new(self.clone())
    }

    fn get_model(&self) -> Box<dyn Model> {
        Box::new(self.backup_model.clone())
    }

    fn render(&self, frame: &mut Frame) {
        let databse_configs = self.backup_model.configs.get_database_configs();
        let storage_configs = self.backup_model.configs.get_storage_configs();

        if self.backup_model.in_progress {
            let block = Block::new()
                .title("Backup in progress")
                .borders(Borders::all())
                .border_set(symbols::border::ROUNDED);

            let selected_database_config = databse_configs
                .iter()
                .find(|config| Some(config.id.clone()) == self.backup_model.selected_database_id);

            let selected_storage_config = storage_configs.iter().find(|config| {
                let config_id = match config {
                    StorageConfig::Local(config) => &config.id,
                    StorageConfig::S3(config) => &config.id,
                };

                Some(config_id.clone()) == self.backup_model.selected_storage_id
            });

            if let Some(database_config) = selected_database_config
                && let Some(storage_config) = selected_storage_config
            {
                let storage_name = match storage_config {
                    StorageConfig::Local(config) => &config.name,
                    StorageConfig::S3(config) => &config.name,
                };

                let text = format!(
                    "Dumping \"{}\" database in \"{}\" storage...",
                    database_config.name, storage_name
                );

                let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
                frame.render_widget(paragraph, frame.area());
                return;
            }
        }

        let [column1, column2] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .flex(Flex::Center)
                .areas(frame.area());

        let database_items: Vec<ListItem> = databse_configs
            .iter()
            .map(|config| {
                let active = matches!(self.backup_model.selection_mode, SelectionMode::DB);
                let highlighted = self.backup_model.highlight_database_id == config.id && active;
                let selected = if let Some(selected_id) = &self.backup_model.selected_database_id {
                    config.id == *selected_id
                } else {
                    false
                };

                ListItem {
                    label: config.name.clone(),
                    highlighted,
                    selected,
                }
            })
            .collect();

        let block = Block::new()
            .title("Select Database")
            .borders(Borders::all())
            .border_set(symbols::border::ROUNDED);
        let list = create_list(database_items).block(block);
        frame.render_widget(list, column1);

        let storage_items: Vec<ListItem> = storage_configs
            .iter()
            .map(|config| {
                let current_id = match config {
                    StorageConfig::Local(config) => config.id.clone(),
                    StorageConfig::S3(config) => config.id.clone(),
                };

                let current_name = match config {
                    StorageConfig::Local(config) => config.name.clone(),
                    StorageConfig::S3(config) => config.name.clone(),
                };

                let active = matches!(self.backup_model.selection_mode, SelectionMode::Storage);
                let highlighted = self.backup_model.highlight_storage_id == current_id && active;
                let selected = if let Some(selected_id) = &self.backup_model.selected_storage_id {
                    current_id == *selected_id
                } else {
                    false
                };

                ListItem {
                    label: current_name,
                    highlighted,
                    selected,
                }
            })
            .collect();

        let block = Block::new()
            .title("Select Storage")
            .borders(Borders::all())
            .border_set(symbols::border::ROUNDED);
        let list = create_list(storage_items).block(block);
        frame.render_widget(list, column2);
    }
}
