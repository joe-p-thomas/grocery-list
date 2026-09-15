use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let suggestions_height = if app.suggestions.is_empty() {
        0
    } else {
        app.suggestions.len().min(5) as u16 + 2
    };

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(suggestions_height),
            Constraint::Length(3),
        ])
        .split(frame.area());

    frame.render_widget(Paragraph::new("grocery-list").bold(), root[0]);

    let log_items: Vec<ListItem> = app
        .log
        .iter()
        .map(|line| ListItem::new(line.as_str()))
        .collect();
    frame.render_widget(
        List::new(log_items).block(Block::default().borders(Borders::ALL).title("Output")),
        root[1],
    );

    if !app.suggestions.is_empty() {
        let items: Vec<ListItem> = app
            .suggestions
            .iter()
            .map(|command| ListItem::new(*command))
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Suggestions (\u{2191}\u{2193} to select, Tab to complete)"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut state = ListState::default().with_selected(Some(app.selected_suggestion));
        frame.render_stateful_widget(list, root[2], &mut state);
    }

    frame.render_widget(
        Paragraph::new(app.input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Command")),
        root[3],
    );

    frame.set_cursor_position((root[3].x + 1 + app.input.len() as u16, root[3].y + 1));
}
