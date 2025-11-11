use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
};

use crate::{
    app::{App, CurrentView},
    database::view::DatabaseView,
    home::view::HomeView,
};

pub fn ui(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(50), Constraint::Length(50)])
        .split(frame.area());

    match &app.current_view {
        CurrentView::Home(home_model) => {
            let view = HomeView::new(home_model.clone());
            view.render(frame);
        }
        CurrentView::Database(database_screen) => {
            let view = DatabaseView::new(database_screen.clone());
            view.render(frame);
        }
        CurrentView::Exiting => {
            let span = Span::styled("Exiting", Style::default().fg(Color::Green));
            frame.render_widget(span, layout[0]);
        }
    }
}
