use ratatui::{
    layout::{Constraint, Direction, Layout},
    symbols,
    widgets::{Block, Borders},
};

use crate::tui::{
    model::Model,
    storage::model::{CurrentInput, StorageModel},
    utils::{ListItem, create_list, render_input},
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
            .title("Select storage type")
            .borders(Borders::all())
            .border_set(symbols::border::ROUNDED);

        let items: Vec<ListItem> = self
            .storage_model
            .storage_type_options
            .iter()
            .map(|it| {
                let highlighted = (self.storage_model.highlighted_option_index as usize)
                    == self
                        .storage_model
                        .storage_type_options
                        .iter()
                        .position(|x| x == it)
                        .unwrap();

                ListItem {
                    label: it.clone(),
                    highlighted,
                    selected: false,
                }
            })
            .collect();

        let list = create_list(items).block(block);
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

#[derive(Clone, Debug)]
pub struct S3StorageView {
    storage_model: StorageModel,
}

impl S3StorageView {
    pub fn new(storage_model: StorageModel) -> Self {
        S3StorageView { storage_model }
    }
}

impl View for S3StorageView {
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
            .s3_input_location
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
            &self.storage_model.s3_input_location,
            "Location",
            matches!(&self.storage_model.current_input, CurrentInput::S3Location),
            inputs_layout[1],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.storage_model.s3_input_bucket,
            "Bucket",
            matches!(&self.storage_model.current_input, CurrentInput::S3Bucket),
            inputs_layout[2],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.storage_model.s3_input_region,
            "Region",
            matches!(&self.storage_model.current_input, CurrentInput::S3Region),
            inputs_layout[3],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.storage_model.s3_input_endpoint,
            "Endpoint",
            matches!(&self.storage_model.current_input, CurrentInput::S3Endpoint),
            inputs_layout[4],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.storage_model.s3_input_access_key,
            "Access Key",
            matches!(&self.storage_model.current_input, CurrentInput::S3AccessKey),
            inputs_layout[5],
            scroll,
            false,
        );

        render_input(
            frame,
            &self.storage_model.s3_input_secret_key,
            "Secret Key",
            matches!(&self.storage_model.current_input, CurrentInput::S3SecretKey),
            inputs_layout[6],
            scroll,
            false,
        );
    }
}
