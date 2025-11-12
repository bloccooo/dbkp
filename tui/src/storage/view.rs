use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem},
};

use crate::{
    model::Model,
    storage::model::{CurrentInput, StorageModel},
    utils::render_input,
    view::View,
};

#[derive(Clone, Debug)]
pub struct StorageView {
    storage_model: StorageModel,
}

impl StorageView {
    pub fn new(storage_model: StorageModel) -> Self {
        StorageView { storage_model }
    }
}

impl View for StorageView {
    fn clone_box(&self) -> Box<dyn View> {
        Box::new(self.clone())
    }

    fn get_model(&self) -> Box<dyn Model> {
        Box::new(self.storage_model.clone())
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let block = Block::new()
            .borders(Borders::all())
            .border_set(symbols::border::ROUNDED);

        let items: Vec<ListItem> = self
            .storage_model
            .storage_type_options
            .iter()
            .map(|it| {
                if (self.storage_model.highlighted_option_index as usize)
                    == self
                        .storage_model
                        .storage_type_options
                        .iter()
                        .position(|x| x == it)
                        .unwrap()
                {
                    ListItem::from(it.as_str()).style(Style::default().bg(Color::LightBlue))
                } else {
                    ListItem::from(it.as_str())
                }
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_widget(list, frame.area());
    }
}

#[derive(Clone, Debug)]
pub struct LocalStorageView {
    storage_model: StorageModel,
}

impl LocalStorageView {
    pub fn new(storage_model: StorageModel) -> Self {
        LocalStorageView { storage_model }
    }
}

impl View for LocalStorageView {
    fn clone_box(&self) -> Box<dyn View> {
        Box::new(self.clone())
    }

    fn get_model(&self) -> Box<dyn Model> {
        Box::new(self.storage_model.clone())
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let inputs_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let width = inputs_layout[0].width.max(3) - 3;
        let scroll = self
            .storage_model
            .local_input_location
            .visual_scroll(width as usize);

        render_input(
            frame,
            &self.storage_model.input_config_name,
            "Config Name",
            matches!(&self.storage_model.current_input, CurrentInput::ConfigName),
            inputs_layout[0],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.storage_model.local_input_location,
            "Location",
            matches!(
                &self.storage_model.current_input,
                CurrentInput::LocalLocation
            ),
            inputs_layout[1],
            scroll,
            false,
        );
    }
}
