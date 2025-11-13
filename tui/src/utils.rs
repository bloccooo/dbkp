use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, List, ListItem as RatatuiListItem, Paragraph},
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

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub struct ListItem {
    pub label: String,
    pub highlighted: bool,
    pub selected: bool,
}

pub fn create_list(items: Vec<ListItem>) -> List<'static> {
    let items: Vec<RatatuiListItem> = items
        .iter()
        .map(|item| {
            if item.highlighted && item.selected {
                RatatuiListItem::from(format!("{} {}", "✓", item.label))
                    .style(Style::default().bg(Color::LightBlue))
            } else if item.selected {
                RatatuiListItem::from(format!("{} {}", "✓", item.label))
                    .style(Style::default().bg(Color::Gray))
            } else if item.highlighted {
                RatatuiListItem::from(item.label.clone())
                    .style(Style::default().bg(Color::LightBlue))
            } else {
                RatatuiListItem::from(item.label.clone())
            }
        })
        .collect();

    List::new(items)
}
