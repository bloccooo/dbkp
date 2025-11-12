use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Paragraph},
};
use tui_input::Input;

pub fn render_input(
    frame: &mut Frame,
    input: &Input,
    title: &str,
    is_active: bool,
    area: Rect,
    scroll: usize,
    obfuscate: bool,
) {
    let block = Block::new()
        .title(title)
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(if is_active {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default()
        });

    let value = input.value();
    let display_value = if obfuscate {
        "•".repeat(value.len())
    } else {
        value.to_string()
    };

    let paragraph = Paragraph::new(display_value)
        .scroll((0, scroll as u16))
        .block(block);

    frame.render_widget(paragraph, area);

    if is_active {
        let x = input.visual_cursor().max(scroll) - scroll + 1;
        frame.set_cursor_position((area.x + x as u16, area.y + 1));
    }
}
