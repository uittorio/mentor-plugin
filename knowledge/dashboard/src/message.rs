use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Paragraph},
};

use crate::{logger::Log, state::Model};

pub fn render_message(frame: &mut Frame, area: Rect, model: &mut Model) {
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
