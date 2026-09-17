use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::app::{App, Mode};

pub fn draw(frame: &mut Frame, app: &App) {
    let suggestions_height = if app.suggestions.is_empty() {
        0
    } else {
        app.suggestions.len().min(5) as u16 + 2
    };
    let status_height = if app.status.is_some() { 1 } else { 0 };

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(status_height),
            Constraint::Length(suggestions_height),
            Constraint::Length(3),
        ])
        .split(frame.area());

    frame.render_widget(Paragraph::new("grocery-list").bold(), root[0]);

    match app.mode {
        Mode::Command => draw_output(frame, app, root[1]),
        Mode::List | Mode::AddItem => draw_list(frame, app, root[1]),
    }

    if let Some(status) = &app.status {
        let style = if status.to_lowercase().starts_with("unknown") || status.starts_with("couldn't") {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        frame.render_widget(Paragraph::new(status.as_str()).style(style), root[2]);
    }

    if !app.suggestions.is_empty() {
        let items: Vec<ListItem> = app
            .suggestions
            .iter()
            .map(|suggestion| ListItem::new(suggestion.as_str()))
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Suggestions (\u{2191}\u{2193} to select, Tab to complete)"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut state = ListState::default().with_selected(Some(app.selected_suggestion));
        frame.render_stateful_widget(list, root[3], &mut state);
    }

    let input_title = match app.mode {
        Mode::Command => "Command",
        Mode::List => "List command",
        Mode::AddItem => "Add item",
    };
    frame.render_widget(
        Paragraph::new(app.input.as_str())
            .block(Block::default().borders(Borders::ALL).title(input_title)),
        root[4],
    );

    frame.set_cursor_position((root[4].x + 1 + app.input.len() as u16, root[4].y + 1));
}

fn draw_output(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .log
        .iter()
        .map(|line| ListItem::new(line.as_str()))
        .collect();
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("Output")),
        area,
    );
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let grouped = app.grouped_list();

    let items: Vec<ListItem> = if grouped.is_empty() {
        vec![ListItem::new("(no items yet)")]
    } else {
        grouped
            .into_iter()
            .flat_map(|(section, items)| {
                std::iter::once(ListItem::new(section.bold())).chain(items.into_iter().map(
                    |(name, count)| {
                        let line = if count > 1 {
                            format!("  {name} (x{count})")
                        } else {
                            format!("  {name}")
                        };
                        ListItem::new(line)
                    },
                ))
            })
            .collect()
    };

    let title = match app.mode {
        Mode::AddItem => "List (Esc to stop adding)",
        _ => "List (Esc to go back, /add to add items, /clear to empty)",
    };
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
}
