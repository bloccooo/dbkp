use anyhow::Result;
use crossterm::event::Event;

use crate::view::View;

pub trait Model {
    fn run_hook(&mut self) -> Result<Option<Box<dyn View>>>;
    fn handle_event(&mut self, event: &Event) -> Result<Option<Box<dyn View>>>;
}
