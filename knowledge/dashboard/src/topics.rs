use chrono::DateTime;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Cell, Row, Table},
};

use crate::state::Model;

pub fn render_topics(frame: &mut Frame, area: Rect, model: &Model) {
    let now_epoc = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|t| t.as_secs())
        .unwrap();

    let rows = model.topics.iter().map(|t| {
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

    let table = Table::new(
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

    frame.render_widget(table, area);
}
