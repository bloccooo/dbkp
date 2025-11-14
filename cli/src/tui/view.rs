use ratatui::Frame;

use crate::tui::model::Model;

pub trait View: std::fmt::Debug + Send + Sync {
    fn render(&self, frame: &mut Frame);
    fn clone_box(&self) -> Box<dyn View>;
    fn get_model(&self) -> Box<dyn Model>;
}

impl Clone for Box<dyn View> {
    fn clone(&self) -> Box<dyn View> {
        self.clone_box()
    }
}
