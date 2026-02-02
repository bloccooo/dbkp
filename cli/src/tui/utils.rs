use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, List, ListItem as RatatuiListItem, ListState, Paragraph},
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

pub fn create_list(items: Vec<ListItem>, width: u16) -> (List<'static>, ListState) {
    let mut result: Vec<RatatuiListItem> = Vec::new();
    // Account for block borders (2 chars) and some padding
    let separator_width = width.saturating_sub(2) as usize;
    let separator = "─".repeat(separator_width);

    let mut selected_line: Option<usize> = None;

    for (i, item) in items.iter().enumerate() {
        let prefix = if item.highlighted { "● " } else { "  " };
        let checkbox = if item.selected { "✓ " } else { "" };

        // Track which line the highlighted item is on (accounting for separators)
        if item.highlighted {
            selected_line = Some(result.len());
        }

        let list_item = if item.highlighted {
            RatatuiListItem::from(format!("{}{}{}", prefix, checkbox, item.label))
                .style(Style::default().fg(Color::LightBlue))
        } else if item.selected {
            RatatuiListItem::from(format!("{}{}{}", prefix, checkbox, item.label))
                .style(Style::default().fg(Color::Gray))
        } else {
            RatatuiListItem::from(format!("{}{}", prefix, item.label))
        };

        result.push(list_item);

        // Add separator line between items (not after the last one)
        if i < items.len() - 1 {
            result.push(
                RatatuiListItem::from(separator.clone())
                    .style(Style::default().fg(Color::DarkGray)),
            );
        }
    }

    let mut state = ListState::default();
    state.select(selected_line);

    (List::new(result), state)
}
