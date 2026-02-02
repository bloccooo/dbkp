use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, List, ListItem as RatatuiListItem, ListState},
};
use tui_input::Input;

const SPINNER_FRAMES: [char; 4] = ['|', '/', '-', '\\'];

/// Returns the current spinner character based on system time
pub fn spinner() -> char {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let frame_index = (millis / 100) as usize % SPINNER_FRAMES.len();
    SPINNER_FRAMES[frame_index]
}

/// Formats a byte count into a human-readable string (e.g., "1.5 MB")
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub struct InputItem<'a> {
    pub label: &'a str,
    pub input: &'a Input,
    pub active: bool,
    pub obfuscate: bool,
}

/// Renders a form with multiple inputs in a list-like style.
/// Each input shows a labeled separator line followed by the value.
/// Example:
/// ```
/// ─ Config Name
/// > my config
/// ─ Location
/// > /path/to/backup
/// ```
pub fn render_input_form(frame: &mut Frame, title: &str, items: Vec<InputItem>, area: Rect) {
    let block = Block::new()
        .title(format!(" {} ", title))
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::all())
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(ratatui::widgets::Padding::uniform(1));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Build list items: each input has label separator line + value line
    let mut list_items: Vec<RatatuiListItem> = Vec::new();
    let mut active_line: Option<usize> = None;
    let mut active_input_info: Option<(&Input, usize)> = None;

    for (i, item) in items.iter().enumerate() {
        // Label separator line: "── Label"
        let label_with_spaces = format!(" {} ", item.label);
        let label_line = format!("─{}", label_with_spaces);

        let label_item = if item.active {
            RatatuiListItem::from(label_line).style(Style::default().fg(Color::LightBlue))
        } else {
            RatatuiListItem::from(label_line).style(Style::default().fg(Color::DarkGray))
        };
        list_items.push(label_item);

        // Value line
        let value = item.input.value();
        let display_value = if item.obfuscate {
            "•".repeat(value.len())
        } else {
            value.to_string()
        };

        // Track which line the active input value is on (for scrolling to show the input)
        if item.active {
            active_line = Some(list_items.len());
        }

        let value_line = if item.active {
            // Track cursor position info
            active_input_info = Some((item.input, list_items.len()));
            RatatuiListItem::from(format!("> {}", display_value))
                .style(Style::default().fg(Color::LightBlue))
        } else {
            RatatuiListItem::from(format!("  {}", display_value))
                .style(Style::default().fg(Color::Gray))
        };
        list_items.push(value_line);

        // Add blank line between items (not after the last one)
        if i < items.len() - 1 {
            list_items.push(RatatuiListItem::from(""));
        }
    }

    let list = List::new(list_items);
    let mut state = ListState::default();
    state.select(active_line);

    frame.render_stateful_widget(list, inner_area, &mut state);

    // Set cursor position for active input
    if let Some((input, value_line_idx)) = active_input_info {
        // Calculate the y position based on scroll offset and line index
        let offset = state.offset();
        let visible_line = value_line_idx.saturating_sub(offset);

        // Only show cursor if the value line is visible
        if visible_line < inner_area.height as usize {
            let width = inner_area.width.saturating_sub(4); // Account for "> " prefix and padding
            let scroll = input.visual_scroll(width as usize);
            let cursor_x = input.visual_cursor().max(scroll) - scroll;

            frame.set_cursor_position((
                inner_area.x + 2 + cursor_x as u16, // 2 = "> " prefix
                inner_area.y + visible_line as u16,
            ));
        }
    }
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

    let mut highlighted_line: Option<usize> = None;
    let mut selected_line: Option<usize> = None;

    for (i, item) in items.iter().enumerate() {
        let prefix = if item.highlighted { "● " } else { "  " };
        let checkbox = if item.selected { "✓ " } else { "" };

        // Track which line the highlighted item is on (accounting for separators)
        if item.highlighted {
            highlighted_line = Some(result.len());
        }
        // Also track selected items for scroll position when nothing is highlighted
        if item.selected {
            selected_line = Some(result.len());
        }

        let list_item = if item.highlighted {
            RatatuiListItem::from(format!("{}{}{}", prefix, checkbox, item.label))
                .style(Style::default().fg(Color::LightBlue))
        } else if item.selected {
            RatatuiListItem::from(format!("{}{}{}", prefix, checkbox, item.label))
                .style(Style::default().fg(Color::Green))
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
    // Prefer highlighted line, fall back to selected line to preserve scroll position
    state.select(highlighted_line.or(selected_line));

    (List::new(result), state)
}
