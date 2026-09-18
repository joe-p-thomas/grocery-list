mod app;
mod catalog;
mod list;
mod ui;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::{App, Mode};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if matches!(app.mode, Mode::Menu) {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => app.select_menu_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.select_menu_prev(),
                    KeyCode::Enter => app.confirm_menu(),
                    KeyCode::Esc | KeyCode::Char('q') => app.should_quit = true,
                    _ => {}
                }
            } else if matches!(app.mode, Mode::List) && app.input.is_empty() {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => app.select_list_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.select_list_prev(),
                    KeyCode::Char('r') => app.remove_selected(),
                    KeyCode::Esc => app.back(),
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        app.input_changed();
                    }
                    _ => {}
                }
            } else if matches!(app.mode, Mode::Catalog) && app.input.is_empty() {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => app.select_catalog_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.select_catalog_prev(),
                    KeyCode::Enter => app.confirm_catalog_selection(),
                    KeyCode::Esc => app.back(),
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        app.input_changed();
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Enter => app.submit_input(),
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        app.input_changed();
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        app.input_changed();
                    }
                    KeyCode::Up => app.select_prev_suggestion(),
                    KeyCode::Down => app.select_next_suggestion(),
                    KeyCode::Tab => app.accept_suggestion(),
                    KeyCode::Esc => {
                        if matches!(app.mode, Mode::List | Mode::Catalog) {
                            app.input.clear();
                            app.suggestions.clear();
                        } else {
                            app.back();
                        }
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
