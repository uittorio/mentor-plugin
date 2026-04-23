use std::env::Args;
use std::{
    io::Stdout,
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode};
use learning::libsql::libsql_session_storage::LibsqlSessionStorage;
use learning::libsql::libsql_storage::libsql_connection;
use learning::libsql::libsql_topic_storage::LibsqlTopicStorage;
use learning::session_storage::SessionStorage;
use learning::topic_storage::TopicStorage;
use ratatui::layout::{Layout, Offset, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Tabs};
use ratatui::{Frame, Terminal, layout::Constraint, prelude::CrosstermBackend, style::Style};

use crate::sessions::render_sessions;
use crate::state::{Message, Model, View, update};
use crate::topics::render_topics;

mod sessions;
mod state;
mod topics;

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
    let rt = tokio::runtime::Runtime::new()?;
    let conn = rt.block_on(libsql_connection())?;
    let topic_storage = rt.block_on(LibsqlTopicStorage::init(conn.clone()))?;
    let session_storage = rt.block_on(LibsqlSessionStorage::init(conn))?;

    let mut model = Model::new();

    update(
        &mut model,
        Message::UpdateTopics(rt.block_on(topic_storage.get_all())?),
    );
    update(
        &mut model,
        Message::UpdateSessions(rt.block_on(session_storage.get_all())?),
    );

    let mut current_time = Instant::now();

    loop {
        if current_time.elapsed() >= Duration::from_secs(5) {
            update(
                &mut model,
                Message::UpdateTopics(rt.block_on(topic_storage.get_all())?),
            );
            update(
                &mut model,
                Message::UpdateSessions(rt.block_on(session_storage.get_all())?),
            );
            current_time = Instant::now();
        };

        terminal.draw(|frame| render(frame, &mut model))?;
        let event = crossterm::event::poll(Duration::from_millis(50))?;

        match event {
            true => match crossterm::event::read()? {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('s') => update(&mut model, Message::ShowSessionView),
                    KeyCode::Char('t') => update(&mut model, Message::ShowTopicView),
                    KeyCode::Char('j') => update(&mut model, Message::NavigateDown),
                    KeyCode::Char('k') => update(&mut model, Message::NavigateUp),
                    KeyCode::Char('h') => update(&mut model, Message::PrevPane),
                    KeyCode::Char('l') => update(&mut model, Message::NextSessionPane),
                    KeyCode::Esc => update(&mut model, Message::ResetFilters),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, model: &mut Model) {
    let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
    let [top, main] = frame.area().layout(&layout);

    let title = Line::from_iter([
        Span::from("Mentor dashboard").bold(),
        Span::from(
            "((q) to quit, (s) sessions, (t) topics, (j,k) navigate up and down, (h,l) rotate pane), (esc) to reset category filter",
        ),
    ]);

    frame.render_widget(title.centered(), top);

    render_content(frame, main, model);
    render_tabs(frame, main + Offset::new(1, 0), model);
}

fn render_content(frame: &mut Frame, area: Rect, model: &mut Model) {
    let block = Block::bordered();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match model.selected_view {
        View::Topics => render_topics(frame, inner, model),
        View::Sessions => render_sessions(frame, inner, model),
    };
}

fn render_tabs(frame: &mut Frame, area: Rect, model: &Model) {
    let [_, center, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(30),
        Constraint::Fill(1),
    ])
    .areas(area);

    let tabs = Tabs::new(vec!["Topics", "Sessions"])
        .style(Color::White)
        .highlight_style(Style::default().magenta().on_black().bold())
        .select(model.selected_view as usize)
        .divider("|")
        .padding(" ", " ");
    frame.render_widget(tabs, center);
}

fn has_version_argument(args: &mut Args) -> bool {
    return args.any(|a| a == "--version" || a == "-v");
}

fn print_version() -> () {
    let version = env!("CARGO_PKG_VERSION");
    println!("{}", version);
}
