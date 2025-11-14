use dbkp_core::storage::provider::StorageConfig;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    symbols,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tui::{
    model::Model,
    restore::model::{RestoreModel, SelectionMode},
    utils::{ListItem, create_list},
    view::View,
};

#[derive(Clone, Debug)]
pub struct RestoreView {
    pub model: RestoreModel,
}

impl RestoreView {
    pub fn new(model: RestoreModel) -> Self {
        Self { model }
    }
}

impl View for RestoreView {
    fn clone_box(&self) -> Box<dyn View> {
        Box::new(self.clone())
    }

    fn get_model(&self) -> Box<dyn Model> {
        Box::new(self.model.clone())
    }

    fn render(&self, frame: &mut Frame) {
        let databse_configs = self.model.configs.get_database_configs();

        if self.model.in_progress {
            let block = Block::new()
                .title("Restore in progress")
                .borders(Borders::all())
                .border_set(symbols::border::ROUNDED);

            let selected_database_config = databse_configs
                .iter()
                .find(|config| Some(config.id.clone()) == self.model.selected_database_id);

            if let Some(database_config) = selected_database_config
                && let Some(backup_name) = &self.model.selected_backup_id
            {
                let text = format!(
                    "Restoring \"{}\" database with \"{}\" backup...",
                    database_config.name, backup_name
                );

                let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
                frame.render_widget(paragraph, frame.area());
                return;
            }
        }

        if self.model.in_progress {
            let paragraph = Paragraph::new("Loading...").wrap(Wrap { trim: true });
            frame.render_widget(paragraph, frame.area());
            return;
        }

        let [row1, row2] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .flex(Flex::Center)
                .areas(frame.area());

        let [column1, column2] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .flex(Flex::Center)
                .areas(row1);

        let block = Block::new().title("Select Storage").borders(Borders::all());

        let storage_items: Vec<ListItem> = self
            .model
            .configs
            .get_storage_configs()
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

                let active = matches!(self.model.selection_mode, SelectionMode::Storage);
                let highlighted = self.model.highlight_storage_id == current_id && active;
                let selected = if let Some(selected_id) = &self.model.selected_storage_id {
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

        let storage_list = create_list(storage_items).block(block);
        frame.render_widget(storage_list, column1);

        let block = Block::new().title("Select Backup").borders(Borders::all());

        let backup_items: Vec<ListItem> = self
            .model
            .backups
            .iter()
            .map(|backup_id| {
                let active = matches!(self.model.selection_mode, SelectionMode::Backup);
                let selected = self.model.selected_backup_id == Some(backup_id.clone());
                let highlighted =
                    self.model.highlighted_backup_id == Some(backup_id.clone()) && active;

                ListItem {
                    label: backup_id.clone(),
                    highlighted,
                    selected,
                }
            })
            .collect();

        let backups_list = create_list(backup_items).block(block.clone());

        if self.model.loading_backups {
            let paragraph = Paragraph::new("Loading entries...")
                .wrap(Wrap { trim: true })
                .block(block);

            frame.render_widget(paragraph, column2);
        } else {
            frame.render_widget(backups_list, column2);
        }

        let block = Block::new()
            .title("Select Target Database")
            .borders(Borders::all());

        let databases_items: Vec<ListItem> = self
            .model
            .configs
            .get_database_configs()
            .iter()
            .map(|config| {
                let active = matches!(self.model.selection_mode, SelectionMode::Database);
                let highlighted = self.model.highlight_database_id == config.id && active;
                let selected = if let Some(selected_id) = &self.model.selected_database_id {
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

        let databases_list = create_list(databases_items).block(block);
        frame.render_widget(databases_list, row2);
    }
}
