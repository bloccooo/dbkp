use dbkp_core::storage::provider::StorageConfig;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph, Wrap},
};

use crate::{
    backup::model::{BackupModel, SelectionMode},
    model::Model,
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
        if self.backup_model.selected_database_id.is_some()
            && self.backup_model.selected_storage_id.is_some()
        {
            let paragraph = Paragraph::new("Loading...").wrap(Wrap { trim: true });
            frame.render_widget(paragraph, frame.area());
            return;
        }

        let [column1, column2] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .flex(Flex::Center)
                .areas(frame.area());

        let databse_configs = self.backup_model.configs.get_database_configs();
        let storage_configs = self.backup_model.configs.get_storage_configs();

        let database_items: Vec<ListItem> = databse_configs
            .iter()
            .map(|config| {
                let is_highlighted = self.backup_model.highlight_database_id == config.id;
                let is_active = matches!(self.backup_model.selection_mode, SelectionMode::DB);
                let is_selected = if let Some(selected_id) = &self.backup_model.selected_database_id
                {
                    config.id == *selected_id
                } else {
                    false
                };

                if is_selected && is_highlighted && is_active {
                    ListItem::from(format!("{} {}", "✓", config.name))
                        .style(Style::default().bg(Color::LightBlue))
                } else if is_selected {
                    ListItem::from(format!("{} {}", "✓", config.name))
                        .style(Style::default().bg(Color::Gray))
                } else if is_highlighted && is_active {
                    ListItem::from(config.name.to_string())
                        .style(Style::default().bg(Color::LightBlue))
                } else {
                    ListItem::from(config.name.to_string())
                }
            })
            .collect();

        let block = Block::new()
            .borders(Borders::all())
            .border_set(symbols::border::ROUNDED);

        let list = List::new(database_items)
            .block(block.clone())
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

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

                let is_highlighted = self.backup_model.highlight_storage_id == current_id;
                let is_active = matches!(self.backup_model.selection_mode, SelectionMode::Storage);
                let is_selected = if let Some(selected_id) = &self.backup_model.selected_storage_id
                {
                    current_id == *selected_id
                } else {
                    false
                };

                if is_selected && is_highlighted && is_active {
                    ListItem::from(format!("{} {}", "✓", current_name))
                        .style(Style::default().bg(Color::LightBlue))
                } else if is_selected {
                    ListItem::from(format!("{} {}", "✓", current_name))
                        .style(Style::default().bg(Color::Gray))
                } else if is_highlighted && is_active {
                    ListItem::from(current_name).style(Style::default().bg(Color::LightBlue))
                } else {
                    ListItem::from(current_name)
                }
            })
            .collect();

        let storage_list = List::new(storage_items)
            .block(block)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_widget(storage_list, column2);
    }
}
