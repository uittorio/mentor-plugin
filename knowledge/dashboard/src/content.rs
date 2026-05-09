use ratatui::Frame;

use crate::{
    logs::render_logs,
    sessions::render_sessions,
    state::{Model, View},
    topics::render_topics,
};

pub fn render_content(frame: &mut Frame, area: ratatui::layout::Rect, model: &mut Model) {
    match model.selected_view {
        View::Topics => render_topics(frame, area, model),
        View::Sessions => render_sessions(frame, area, model),
        View::Logs => render_logs(frame, area, model),
    };
}
