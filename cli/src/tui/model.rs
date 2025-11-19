use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::Event;

#[async_trait]
pub trait Model: Send + Sync + Unpin {
    async fn handle_event(&mut self, event: &Event) -> Result<()>;
}
