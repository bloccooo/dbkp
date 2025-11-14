use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::Event;

use crate::tui::view::View;

#[async_trait]
pub trait Model: Send + Sync + Unpin {
    async fn handle_event(&mut self, event: &Event) -> Result<()>;
    fn get_next_view(&mut self) -> Result<Option<Box<dyn View>>>;
}
