use learning::{session::Session, topic::Topic};
use ratatui::widgets::TableState;

#[derive(Copy, Clone)]
pub enum View {
    Topics,
    Sessions,
}

pub struct Model {
    pub topics: Vec<Topic>,
    pub sessions: Vec<Session>,
    pub selected_view: View,
    pub topics_state: TableState,
    pub session_state: TableState,
}

impl Model {
    pub fn new() -> Self {
        Model {
            selected_view: View::Topics,
            topics: vec![],
            sessions: vec![],
            topics_state: TableState::default(),
            session_state: TableState::default(),
        }
    }
}

pub enum Message {
    NextView,
    PrevView,
    UpdateTopics(Vec<Topic>),
    UpdateSessions(Vec<Session>),
    NavigateUp,
    NavigateDown,
}

pub fn update_selected_table_up(table_state: &mut TableState) {
    match table_state.selected() {
        Some(s) => {
            let up = if s == 0 { 0 } else { s - 1 };
            table_state.select(Some(up))
        }
        None => table_state.select(Some(0)),
    };
}

pub fn update_selected_table_down(table_state: &mut TableState, list_len: usize) {
    match table_state.selected() {
        Some(s) => {
            let down = if s != list_len - 1 { s + 1 } else { s };
            table_state.select(Some(down))
        }
        None => table_state.select(Some(0)),
    };
}
pub fn update(model: &mut Model, msg: Message) -> () {
    match msg {
        Message::NextView => match model.selected_view {
            View::Topics => model.selected_view = View::Sessions,
            View::Sessions => model.selected_view = View::Topics,
        },
        Message::PrevView => match model.selected_view {
            View::Topics => model.selected_view = View::Sessions,
            View::Sessions => model.selected_view = View::Topics,
        },
        Message::UpdateTopics(topics) => {
            model.topics = topics;
        }
        Message::UpdateSessions(sessions) => {
            model.sessions = sessions;
        }
        Message::NavigateUp => match model.selected_view {
            View::Topics => {
                update_selected_table_up(&mut model.topics_state);
            }
            View::Sessions => {
                update_selected_table_up(&mut model.session_state);
            }
        },
        Message::NavigateDown => match model.selected_view {
            View::Topics => {
                update_selected_table_down(&mut model.topics_state, model.topics.len());
            }
            View::Sessions => {
                update_selected_table_down(&mut model.session_state, model.sessions.len());
            }
        },
    }
}
