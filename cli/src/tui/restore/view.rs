use dbkp_core::storage::provider::StorageConfig;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use crate::tui::{
    model::Model,
    restore::model::{RestoreModel, RestoreStage, SelectionMode},
    utils::{ListItem, create_list, spinner},
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
        let database_configs = self.model.configs.get_database_configs();

        if self.model.in_progress {
            let block = Block::new()
                .title(" Restore in progress ")
                .title_style(Style::default().fg(Color::White))
                .borders(Borders::all())
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(Color::DarkGray))
                .padding(Padding::uniform(1));

            let selected_database_config = database_configs
                .iter()
                .find(|config| Some(config.id.clone()) == self.model.selected_database_id);

            if let Some(database_config) = selected_database_config
                && let Some(backup_name) = &self.model.selected_backup_id
            {
                let text = format!(
                    "{} Restoring \"{}\" database with \"{}\" backup...",
                    spinner(),
                    database_config.name,
                    backup_name
                );

                let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
                frame.render_widget(paragraph, frame.area());
                return;
            }
        }

        match self.model.restore_stage {
            RestoreStage::Selection => {
                let [row1, row2] =
                    Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .flex(Flex::Center)
                        .areas(frame.area());

                let [column1, column2] =
                    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .flex(Flex::Center)
                        .areas(row1);

                let block = Block::new()
                    .title(" Select Storage ")
                    .title_style(Style::default().fg(Color::White))
                    .borders(Borders::all())
                    .border_set(symbols::border::ROUNDED)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .padding(Padding::uniform(1));

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

                let (storage_list, mut state) = create_list(storage_items, column1.width);
                let storage_list = storage_list.block(block);
                frame.render_stateful_widget(storage_list, column1, &mut state);

                let block = Block::new()
                    .title(" Select Backup ")
                    .title_style(Style::default().fg(Color::White))
                    .borders(Borders::all())
                    .border_set(symbols::border::ROUNDED)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .padding(Padding::uniform(1));

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

                let (backups_list, mut backups_state) = create_list(backup_items, column2.width);
                let backups_list = backups_list.block(block.clone());

                if self.model.loading_backups {
                    let text = format!("{} Loading entries...", spinner());
                    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);

                    frame.render_widget(paragraph, column2);
                } else {
                    frame.render_stateful_widget(backups_list, column2, &mut backups_state);
                }

                let block = Block::new()
                    .title(" Select Target Database ")
                    .title_style(Style::default().fg(Color::White))
                    .borders(Borders::all())
                    .border_set(symbols::border::ROUNDED)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .padding(Padding::uniform(1));

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

                let (databases_list, mut db_state) = create_list(databases_items, row2.width);
                let databases_list = databases_list.block(block);
                frame.render_stateful_widget(databases_list, row2, &mut db_state);
            }
            RestoreStage::RestoreConfig => {
                let block = Block::new()
                    .title(" Drop database first ")
                    .title_style(Style::default().fg(Color::White))
                    .borders(Borders::all())
                    .border_set(symbols::border::ROUNDED)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .padding(Padding::uniform(1));

                let items: Vec<ListItem> = vec![
                    ListItem {
                        label: "False".into(),
                        highlighted: !self.model.drop_database,
                        selected: false,
                    },
                    ListItem {
                        label: "True".into(),
                        highlighted: self.model.drop_database,
                        selected: false,
                    },
                ];

                let (databases_list, mut state) = create_list(items, frame.area().width);
                let databases_list = databases_list.block(block);
                frame.render_stateful_widget(databases_list, frame.area(), &mut state);
            }
            RestoreStage::Confirm => {
                let storage_configs = self.model.configs.get_storage_configs();

                let selected_storage_config = storage_configs.iter().find(|config| {
                    let config_id = match config {
                        StorageConfig::Local(config) => &config.id,
                        StorageConfig::S3(config) => &config.id,
                    };
                    Some(config_id.clone()) == self.model.selected_storage_id
                });

                let selected_database_config = database_configs
                    .iter()
                    .find(|config| Some(config.id.clone()) == self.model.selected_database_id);

                if let Some(storage_config) = selected_storage_config
                    && let Some(database_config) = selected_database_config
                    && let Some(backup_name) = &self.model.selected_backup_id
                {
                    let storage_name = match storage_config {
                        StorageConfig::Local(config) => &config.name,
                        StorageConfig::S3(config) => &config.name,
                    };

                    let block = Block::new()
                        .title(" Confirm Restore ")
                        .title_style(Style::default().fg(Color::White))
                        .borders(Borders::all())
                        .border_set(symbols::border::ROUNDED)
                        .border_style(Style::default().fg(Color::Rgb(255, 165, 0)))
                        .padding(Padding::uniform(1));

                    let drop_db_text = if self.model.drop_database {
                        "Yes"
                    } else {
                        "No"
                    };

                    let text = format!(
                        "Storage: {}\nBackup: {}\nTarget Database: {}\nDrop database first: {}\n\nPress Enter to start restore, Esc to cancel.",
                        storage_name, backup_name, database_config.name, drop_db_text
                    );

                    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
                    frame.render_widget(paragraph, frame.area());
                }
            }
        }
    }
}
