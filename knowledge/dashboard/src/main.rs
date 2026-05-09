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
use crate::logger::{DashboardLogger, Log};
use crate::logs::render_logs;
use crate::sessions::render_sessions;
use crate::state::{Message, Model, UpdateCommand, View, update};
use crate::topics::render_topics;

mod config;
mod logger;
mod logs;
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
    let arc_conn = Arc::new(conn);
    let topic_storage = SqlTopicStorage(arc_conn.clone());
    let session_storage = SqlSessionStorage(arc_conn.clone());
    let config = config()?;

    let mut model = Model::new(config, &mut logger);

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

fn render_message(frame: &mut Frame, area: Rect, model: &mut Model) {
    let block = Block::bordered();
    let inner = block.inner(area);

    let first_log_message = model
        .logger
        .logs
        .iter()
        .nth(0)
        .map_or("".to_string(), |l| match l {
            Log::Info(m, _) => m.clone(),
        });

    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(first_log_message), inner);
}

struct KeyConfiguration {
    key: String,
    message: String,
}

fn render_header(frame: &mut Frame, area: Rect, model: &mut Model) {
    let mut configuration = vec![KeyConfiguration {
        key: "q".to_string(),
        message: "quit".to_string(),
    }];

    let mut configuration_topic = vec![
        KeyConfiguration {
            key: "s".to_string(),
            message: "sessions".to_string(),
        },
        KeyConfiguration {
            key: "l".to_string(),
            message: "logs".to_string(),
        },
        KeyConfiguration {
            key: "→|←".to_string(),
            message: "switch pane".to_string(),
        },
        KeyConfiguration {
            key: "Esc".to_string(),
            message: "reset category filter".to_string(),
        },
    ];

    if let Some(config) = model.selected_review_topic_command {
        let name = &model.config.as_ref().unwrap().review_topic_commands[config].name;
        configuration_topic.push(KeyConfiguration {
            key: "a".to_string(),
            message: format!("review topic configuration ({}),", name),
        });
    };

    let mut configuration_session = vec![
        KeyConfiguration {
            key: "t".to_string(),
            message: "topics".to_string(),
        },
        KeyConfiguration {
            key: "l".to_string(),
            message: "logs".to_string(),
        },
    ];

    let mut configuration_logs = vec![
        KeyConfiguration {
            key: "t".to_string(),
            message: "topics".to_string(),
        },
        KeyConfiguration {
            key: "s".to_string(),
            message: "sessions".to_string(),
        },
    ];

    match model.selected_view {
        View::Topics => configuration.append(&mut configuration_topic),
        View::Sessions => configuration.append(&mut configuration_session),
        View::Logs => configuration.append(&mut configuration_logs),
    };

    let instructions = configuration
        .iter()
        .map(|c| format!("({}) {}", c.key, c.message))
        .collect::<Vec<_>>()
        .join(", ");

    let instructions = Paragraph::new(instructions);
    frame.render_widget(instructions.centered(), area);
    render_tabs(frame, area + Offset::new(0, 2), model);
}

fn render_content(frame: &mut Frame, area: Rect, model: &mut Model) {
    match model.selected_view {
        View::Topics => render_topics(frame, area, model),
        View::Sessions => render_sessions(frame, area, model),
        View::Logs => render_logs(frame, area, model),
    };
}

fn render_tabs(frame: &mut Frame, area: Rect, model: &Model) {
    let [_, center, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(30),
        Constraint::Fill(1),
    ])
    .areas(area);

    let tabs = Tabs::new(vec!["Topics", "Sessions", "Logs"])
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
