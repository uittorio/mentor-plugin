use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Cell, Padding, Paragraph, Row, Table, Wrap},
};

use crate::state::{Model, SessionsPane};

pub fn render_sessions(frame: &mut Frame, area: Rect, model: &mut Model) {
    let rows = model.sessions.iter().map(|s| {
        Row::new([
            Cell::from(s.name.as_str()),
            Cell::from(s.created_at.format("%b %e %T %Y").to_string()),
        ])
    });

    let header = Row::new(vec![Cell::from("Session"), Cell::from("Created")])
        .style(Style::new().bold())
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [Constraint::Percentage(60), Constraint::Percentage(40)],
    )
    .row_highlight_style(Style::new().bg(Color::DarkGray))
    .header(header);

    let layout =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).spacing(1);

    let [left, right] = area.layout(&layout);

    let left_block = match model.selected_session_pane {
        SessionsPane::List => Block::bordered().border_style(Style::new().fg(Color::Blue)),
        SessionsPane::SessionMd => Block::bordered().border_style(Style::new().fg(Color::Reset)),
    };

    let left_inner = left_block.inner(left);

    frame.render_widget(left_block, left);

    frame.render_stateful_widget(&table, left_inner, &mut model.session_state);

    match model.session_state.selected() {
        Some(s) => {
            let block = match model.selected_session_pane {
                SessionsPane::List => Block::bordered()
                    .border_style(Style::new().fg(Color::Reset))
                    .padding(Padding::uniform(2)),
                SessionsPane::SessionMd => Block::bordered()
                    .padding(Padding::uniform(2))
                    .border_style(Style::new().fg(Color::Blue)),
            };

            let inner = block.inner(right);

            frame.render_widget(block, right);

            match &model.sessions[s].content {
                Some(file_content) => frame.render_widget(
                    Paragraph::new(file_content.as_str())
                        .scroll((model.session_md_scroll, 0))
                        .wrap(Wrap { trim: false }),
                    inner,
                ),
                None => {
                    frame.render_widget(Paragraph::new("No content available"), inner);
                }
            }
        }
        None => (),
    }
}
