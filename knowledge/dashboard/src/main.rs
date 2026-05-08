use std::env::Args;
use std::sync::Arc;
use std::{
    io::Stdout,
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode};
use learning::session_storage::SessionStorage;
use learning::sql::sql_session_storage::SqlSessionStorage;
use learning::sql::sql_storage::SqlConnection;
use learning::sql::sql_topic_storage::SqlTopicStorage;
use learning::topic_storage::TopicStorage;
use ratatui::layout::{Layout, Offset, Rect};
use ratatui::style::Color;
use ratatui::widgets::{Block, Paragraph, Tabs};
use ratatui::{Frame, Terminal, layout::Constraint, prelude::CrosstermBackend, style::Style};

use crate::config::config;
use crate::sessions::render_sessions;
use crate::state::{Message, Model, UpdateCommand, View, update};
use crate::topics::render_topics;

mod config;
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
    let conn = rt.block_on(SqlConnection::new())?;
    let arc_conn = Arc::new(conn);
    let topic_storage = SqlTopicStorage(arc_conn.clone());
    let session_storage = SqlSessionStorage(arc_conn.clone());
    let config = config()?;

    let mut model = Model::new(config);

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
                    KeyCode::Char('a') => update(&mut model, Message::NextReviewTopicCommand),
                    KeyCode::Enter => match update(&mut model, Message::ReviewTopic) {
                        Some(UpdateCommand::StartReview(topic_name)) => {
                            match (&model.config, &model.selected_review_topic_command) {
                                (Some(config), Some(index)) => {
                                    let command = &config.review_topic_commands[*index];

                                    command.start(&topic_name)?;
                                }
                                (_, _) => (),
                            }

                            None
                        }
                        None => None,
                    },
                    KeyCode::Esc => update(&mut model, Message::ResetFilters),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };

        ()
    }

    Ok(())
}

fn render(frame: &mut Frame, model: &mut Model) {
    let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
    let [top, main] = frame.area().layout(&layout);

    let configuration = if let Some(config) = model.selected_review_topic_command {
        let name = &model.config.as_ref().unwrap().review_topic_commands[config].name;
        format!("(a) to rotate review topic configuration ({}),", name)
    } else {
        "".to_string()
    };

    let instructions = Paragraph::new(format!(
        "(q) to quit, (s) sessions, (t) topics, (j,k) navigate up and down, (h,l) rotate pane, {} (esc) to reset category filter",
        configuration
    ));

    frame.render_widget(instructions.centered(), top);

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
