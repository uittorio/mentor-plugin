use std::env::Args;
use std::{
    io::Stdout,
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode};
use learning::session_storage::SessionStorage;
use learning::sqlite::sqlite_session_storage::SqliteSessionStorage;
use learning::sqlite::sqlite_topic_storage::SqliteTopicStorage;
use learning::topic_storage::TopicStorage;
use ratatui::layout::{Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Block;
use ratatui::{Frame, Terminal, layout::Constraint, prelude::CrosstermBackend};

use crate::sessions::render_sessions;
use crate::state::{
    Message, Model, View, Zone, navigate_down, navigate_left, navigate_right, navigate_up,
    update,
};
use crate::topics_view::{epoch_now, render_topics_view};

mod sessions;
mod state;
mod topics_view;

fn main() -> color_eyre::Result<()> {
    let mut args = std::env::args();
    if has_version_argument(&mut args) {
        print_version();
        return Ok(());
    }
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    app(&mut terminal)?;
    ratatui::restore();
    Ok(())
}

fn app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> color_eyre::Result<()> {
    let topic_storage = SqliteTopicStorage::init()?;
    let session_storage = SqliteSessionStorage::init()?;

    let mut model = Model::new();
    update(&mut model, Message::UpdateTopics(topic_storage.get_all()?));
    update(&mut model, Message::UpdateSessions(session_storage.get_all()?));

    let mut last_refresh = Instant::now();

    loop {
        if last_refresh.elapsed() >= Duration::from_secs(5) {
            update(&mut model, Message::UpdateTopics(topic_storage.get_all()?));
            update(&mut model, Message::UpdateSessions(session_storage.get_all()?));
            last_refresh = Instant::now();
        }

        terminal.draw(|frame| render(frame, &mut model))?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = crossterm::event::read()? {
                match model.selected_view {
                    View::Topics => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('s') => update(&mut model, Message::ShowSessionView),
                        KeyCode::Char('1') => update(&mut model, Message::FocusZone(Zone::Stats)),
                        KeyCode::Char('2') => {
                            update(&mut model, Message::FocusZone(Zone::Categories))
                        }
                        KeyCode::Char('3') => update(&mut model, Message::FocusZone(Zone::Topics)),
                        KeyCode::Esc => update(&mut model, Message::ExitZone),
                        KeyCode::Char('r') => update(&mut model, Message::ResetFilters),
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            update(&mut model, Message::ToggleFilter)
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            let now = epoch_now();
                            navigate_down(&mut model, now);
                        }
                        KeyCode::Char('k') | KeyCode::Up => navigate_up(&mut model),
                        KeyCode::Char('h') | KeyCode::Left => navigate_left(&mut model),
                        KeyCode::Char('l') | KeyCode::Right => navigate_right(&mut model),
                        _ => {}
                    },
                    View::Sessions => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('t') => update(&mut model, Message::ShowTopicView),
                        KeyCode::Char('j') | KeyCode::Down => {
                            navigate_down(&mut model, epoch_now())
                        }
                        KeyCode::Char('k') | KeyCode::Up => navigate_up(&mut model),
                        KeyCode::Char('h') | KeyCode::Left => {
                            update(&mut model, Message::PrevPane)
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            update(&mut model, Message::NextPane)
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, model: &mut Model) {
    let [header, main] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

    let (hint, view_indicator) = match model.selected_view {
        View::Topics => (
            " (q) quit · (s) sessions · (1) stats · (2) categories · (3) topics · (Space) filter · (r) reset · (Esc) unfocus",
            " Topics ",
        ),
        View::Sessions => (
            " (q) quit · (t) topics · (j/k) navigate · (h/l) pane",
            " Sessions ",
        ),
    };

    let title = Line::from_iter([
        Span::from("Mentor").bold(),
        Span::from(view_indicator).magenta().bold(),
        Span::from(hint),
    ]);
    frame.render_widget(title, header);

    render_content(frame, main, model);
}

fn render_content(frame: &mut Frame, area: Rect, model: &mut Model) {
    let block = Block::bordered();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match model.selected_view {
        View::Topics => render_topics_view(frame, inner, model),
        View::Sessions => render_sessions(frame, inner, model),
    }
}

fn has_version_argument(args: &mut Args) -> bool {
    args.any(|a| a == "--version" || a == "-v")
}

fn print_version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}
