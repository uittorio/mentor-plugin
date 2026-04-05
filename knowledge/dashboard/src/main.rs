use std::{
    io::Stdout,
    time::{Duration, Instant},
};

use chrono::DateTime;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    Frame, Terminal,
    layout::Constraint,
    prelude::CrosstermBackend,
    style::Style,
    widgets::{Cell, Row, Table},
};
use topic::{Topic, sqlite_topic_storage::SqliteTopicStorage, topic_storage::TopicStorage};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    app(&mut terminal)?;
    ratatui::restore();
    Ok(())
}

fn app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> color_eyre::Result<()> {
    let storage = SqliteTopicStorage::init()?;
    let mut topics = storage.get_all()?;
    let mut current_time = Instant::now();

    loop {
        if current_time.elapsed() >= Duration::from_secs(5) {
            topics = storage.get_all()?;
            current_time = Instant::now();
        };

        terminal.draw(|frame| render(frame, &topics))?;
        let event = crossterm::event::poll(Duration::from_millis(50));

        match event {
            Ok(true) => match crossterm::event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => break,
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, topics: &Vec<Topic>) {
    let now_epoc = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|t| t.as_secs())
        .unwrap();

    let rows = topics.iter().map(|t| {
        let next_review = t.reviewed_at + (t.interval * 60 * 60 * 24) as u64;
        let next_review_value = DateTime::from_timestamp_secs(next_review as i64)
            .unwrap()
            .format("%b %e %T %Y")
            .to_string();

        let next_review_formatted = if next_review < now_epoc {
            format!("{} (overdue)", next_review_value)
        } else {
            next_review_value
        };
        Row::new([
            Cell::from(t.name.as_str()),
            Cell::from(t.ease_factor.to_string()),
            Cell::from(t.repetitions.to_string()),
            Cell::from(
                DateTime::from_timestamp_secs(t.reviewed_at as i64)
                    .unwrap()
                    .format("%b %e %T %Y")
                    .to_string(),
            ),
            Cell::from(next_review_formatted),
        ])
    });

    let header = Row::new(vec![
        Cell::from("Topic"),
        Cell::from("Ease factor"),
        Cell::from("Repetitions"),
        Cell::from("Last review"),
        Cell::from("Next review"),
    ])
    .style(Style::new().bold())
    .bottom_margin(1);
    let t = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(9),
            Constraint::Percentage(9),
            Constraint::Percentage(18),
            Constraint::Percentage(24),
        ],
    )
    .header(header);
    frame.render_widget(t, frame.area());
}
