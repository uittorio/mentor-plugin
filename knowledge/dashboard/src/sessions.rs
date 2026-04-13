use chrono::DateTime;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Cell, Row, Table},
};

use crate::state::Model;

pub fn render_sessions(frame: &mut Frame, area: Rect, model: &Model) {
    let rows = model.sessions.iter().map(|s| {
        Row::new([
            Cell::from(s.name.as_str()),
            Cell::from(
                DateTime::from_timestamp_secs(s.created_at as i64)
                    .unwrap()
                    .format("%b %e %T %Y")
                    .to_string(),
            ),
        ])
    });

    let header = Row::new(vec![Cell::from("Session"), Cell::from("Created")])
        .style(Style::new().bold())
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [Constraint::Percentage(60), Constraint::Percentage(40)],
    )
    .header(header);

    frame.render_widget(table, area);
}
