use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::app::{App, MENU_ITEMS, Mode};

pub fn draw(frame: &mut Frame, app: &App) {
    let suggestions_height = if app.suggestions.is_empty() {
        0
    } else {
        app.suggestions.len().min(5) as u16 + 2
    };
    let status_height = if app.status.is_some() { 1 } else { 0 };
    let input_height = if matches!(app.mode, Mode::Menu) { 0 } else { 3 };

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(status_height),
            Constraint::Length(suggestions_height),
            Constraint::Length(input_height),
        ])
        .split(frame.area());

    frame.render_widget(Paragraph::new("grocery-list").bold(), root[0]);

    match app.mode {
        Mode::Menu => draw_menu(frame, app, root[1]),
        Mode::List | Mode::AddItem | Mode::RemoveItem => draw_list(frame, app, root[1]),
        Mode::Catalog => draw_catalog(frame, app, root[1]),
        Mode::CatalogSection | Mode::CatalogAddItem => draw_catalog_section(frame, app, root[1]),
    }

    if let Some(status) = &app.status {
        let style = if status.to_lowercase().starts_with("unknown")
            || status.starts_with("couldn't")
            || status.starts_with("No such item")
        {
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

    if !matches!(app.mode, Mode::Menu) {
        let input_title = match app.mode {
            Mode::Menu => "",
            Mode::List => "List command",
            Mode::AddItem => "Add item",
            Mode::RemoveItem => "Remove item",
            Mode::Catalog => "Catalog command",
            Mode::CatalogSection => "Command",
            Mode::CatalogAddItem => "Add item to section",
        };
        frame.render_widget(
            Paragraph::new(app.input.as_str())
                .block(Block::default().borders(Borders::ALL).title(input_title)),
            root[4],
        );

        frame.set_cursor_position((root[4].x + 1 + app.input.len() as u16, root[4].y + 1));
    }
}

fn draw_menu(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .map(|label| ListItem::new(*label))
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .highlight_symbol("> ");
    let mut state = ListState::default().with_selected(Some(app.menu_selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let grouped = app.grouped_list();

    let items: Vec<ListItem> = if grouped.is_empty() {
        vec![ListItem::new("(no items yet)")]
    } else {
        grouped
            .into_iter()
            .flat_map(|(section, items)| {
                std::iter::once(ListItem::new(section.bold()))
                    .chain(items.into_iter().map(|item| ListItem::new(format!("  {item}"))))
            })
            .collect()
    };

    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("List")),
        area,
    );
}

fn draw_catalog(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = if app.catalog_sections.is_empty() {
        vec![ListItem::new("(no catalog loaded)")]
    } else {
        app.catalog_sections
            .iter()
            .map(|section| ListItem::new(section.name.as_str()))
            .collect()
    };

    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("Catalog")),
        area,
    );
}

fn draw_catalog_section(frame: &mut Frame, app: &App, area: Rect) {
    let section = app
        .catalog_sections
        .iter()
        .find(|section| Some(section.name.as_str()) == app.current_section.as_deref());

    let items: Vec<ListItem> = match section {
        Some(section) if !section.items.is_empty() => section
            .items
            .iter()
            .map(|item| ListItem::new(item.as_str()))
            .collect(),
        _ => vec![ListItem::new("(no items)")],
    };

    let title = app.current_section.as_deref().unwrap_or("Section");
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
}
