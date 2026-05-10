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
use ratatui::layout::Layout;
use ratatui::{Frame, Terminal, layout::Constraint, prelude::CrosstermBackend};

use crate::config::config;
use crate::content::render_content;
use crate::header::render_header;
use crate::logger::DashboardLogger;
use crate::message::render_message;
use crate::state::{Message, Model, UpdateCommand, update};

mod config;
mod content;
mod dates;
mod header;
mod logger;
mod logs;
mod message;
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
    let mut logger = DashboardLogger::new();
    let rt = tokio::runtime::Runtime::new()?;
    let conn = rt.block_on(SqlConnection::new())?;
    let connection_type = conn.connection_type.clone();
    let arc_conn = Arc::new(conn);
    let topic_storage = SqlTopicStorage(arc_conn.clone());
    let session_storage = SqlSessionStorage(arc_conn.clone());
    let config = config()?;

    let mut model = Model::new(config, &mut logger, connection_type);

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
                    KeyCode::Char('l') => update(&mut model, Message::ShowLogsView),
                    KeyCode::Down => update(&mut model, Message::NavigateDown),
                    KeyCode::Up => update(&mut model, Message::NavigateUp),
                    KeyCode::Left => update(&mut model, Message::PrevPane),
                    KeyCode::Right => update(&mut model, Message::NextSessionPane),
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
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ]);

    let [top, main, bottom] = frame.area().layout(&layout);

    render_header(frame, top, model);
    render_content(frame, main, model);
    render_message(frame, bottom, model);
}

fn has_version_argument(args: &mut Args) -> bool {
    return args.any(|a| a == "--version" || a == "-v");
}

fn print_version() -> () {
    let version = env!("CARGO_PKG_VERSION");
    println!("{}", version);
}
