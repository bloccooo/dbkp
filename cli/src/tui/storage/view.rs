use ratatui::{
    symbols,
    widgets::{Block, Borders, Padding},
};

use crate::tui::{
    model::Model,
    storage::model::{CurrentInput, StorageModel},
    utils::{InputItem, ListItem, create_list, render_input_form},
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
            .border_set(symbols::border::ROUNDED)
            .padding(Padding::uniform(1));

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

        let (list, mut state) = create_list(items, frame.area().width);
        let list = list.block(block);
        frame.render_stateful_widget(list, frame.area(), &mut state);
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
        let items = vec![
            InputItem {
                label: "Config Name",
                input: &self.storage_model.input_config_name,
                active: matches!(&self.storage_model.current_input, CurrentInput::ConfigName),
                obfuscate: false,
            },
            InputItem {
                label: "Location",
                input: &self.storage_model.local_input_location,
                active: matches!(
                    &self.storage_model.current_input,
                    CurrentInput::LocalLocation
                ),
                obfuscate: false,
            },
        ];

        render_input_form(frame, "Local Storage", items, frame.area());
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
        let items = vec![
            InputItem {
                label: "Config Name",
                input: &self.storage_model.input_config_name,
                active: matches!(&self.storage_model.current_input, CurrentInput::ConfigName),
                obfuscate: false,
            },
            InputItem {
                label: "Location",
                input: &self.storage_model.s3_input_location,
                active: matches!(&self.storage_model.current_input, CurrentInput::S3Location),
                obfuscate: false,
            },
            InputItem {
                label: "Bucket",
                input: &self.storage_model.s3_input_bucket,
                active: matches!(&self.storage_model.current_input, CurrentInput::S3Bucket),
                obfuscate: false,
            },
            InputItem {
                label: "Region",
                input: &self.storage_model.s3_input_region,
                active: matches!(&self.storage_model.current_input, CurrentInput::S3Region),
                obfuscate: false,
            },
            InputItem {
                label: "Endpoint",
                input: &self.storage_model.s3_input_endpoint,
                active: matches!(&self.storage_model.current_input, CurrentInput::S3Endpoint),
                obfuscate: false,
            },
            InputItem {
                label: "Access Key",
                input: &self.storage_model.s3_input_access_key,
                active: matches!(&self.storage_model.current_input, CurrentInput::S3AccessKey),
                obfuscate: false,
            },
            InputItem {
                label: "Secret Key",
                input: &self.storage_model.s3_input_secret_key,
                active: matches!(&self.storage_model.current_input, CurrentInput::S3SecretKey),
                obfuscate: true,
            },
        ];

        render_input_form(frame, "S3 Storage", items, frame.area());
    }
}
