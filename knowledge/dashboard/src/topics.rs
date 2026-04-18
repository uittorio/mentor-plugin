use chrono::DateTime;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Cell, Row, Table},
};

use crate::state::Model;
use learning::topic::QuestionDepth;

pub fn render_topics(frame: &mut Frame, area: Rect, model: &mut Model) {
    let now_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|t| t.as_secs())
        .unwrap_or(0);

    let rows = model.topics.iter().map(|t| {
        let next_review = t.reviewed_at + (t.interval as u64 * 86400);
        let is_overdue = next_review < now_epoch;

        let next_str = {
            let formatted = DateTime::from_timestamp_secs(next_review as i64)
                .unwrap()
                .format("%b %e %T %Y")
                .to_string();
            if is_overdue { format!("{} (overdue)", formatted) } else { formatted }
        };

        let cats = if t.categories.is_empty() {
            "-".to_string()
        } else {
            t.categories.join(", ")
        };

        let row_style = match (is_overdue, t.question_depth()) {
            (true, _) => Style::new().fg(Color::Red),
            (_, QuestionDepth::Skip) => Style::new().fg(Color::Green),
            (_, QuestionDepth::Light) => Style::new().fg(Color::Yellow),
            _ => Style::new(),
        };

        Row::new([
            Cell::from(t.name.clone()),
            Cell::from(cats),
            Cell::from(format!("{:.2}", t.ease_factor)),
            Cell::from(t.repetitions.to_string()),
            Cell::from(
                DateTime::from_timestamp_secs(t.reviewed_at as i64)
                    .unwrap()
                    .format("%b %e %Y")
                    .to_string(),
            ),
            Cell::from(next_str),
        ])
        .style(row_style)
    });

    let header = Row::new(vec![
        Cell::from("Topic"),
        Cell::from("Categories"),
        Cell::from("Ease"),
        Cell::from("Reps"),
        Cell::from("Last review"),
        Cell::from("Next review"),
    ])
    .style(Style::new().bold())
    .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(22),
            Constraint::Percentage(6),
            Constraint::Percentage(5),
            Constraint::Percentage(14),
            Constraint::Percentage(28),
        ],
    )
    .row_highlight_style(Style::new().bg(Color::DarkGray))
    .header(header);

    frame.render_stateful_widget(table, area, &mut model.topics_state);
}
