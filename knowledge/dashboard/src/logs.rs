use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, List},
};

use crate::{logger::Log, state::Model};

pub fn render_logs(frame: &mut Frame, area: Rect, model: &mut Model) {
    let block = Block::bordered().border_style(Style::new().fg(Color::Blue));

    let left_inner = block.inner(area);

    frame.render_widget(block, area);

    let items = model
        .logger
        .logs
        .iter()
        .map(|l| match l {
            Log::Info(_, _) => format!("{}", l),
        })
        .collect::<Vec<_>>();

    let list = List::new(items).highlight_style(Style::new().bg(Color::DarkGray));

    frame.render_stateful_widget(list, left_inner, &mut model.logs_state);
}
