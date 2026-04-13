use learning::{session::Session, topic::Topic};

#[derive(Copy, Clone)]
pub enum View {
    Topics,
    Sessions,
}

pub struct Model {
    pub topics: Vec<Topic>,
    pub sessions: Vec<Session>,
    pub selected_view: View,
}

impl Model {
    pub fn new() -> Self {
        Model {
            selected_view: View::Topics,
            topics: vec![],
            sessions: vec![],
        }
    }
}

pub enum Message {
    NextView,
    PrevView,
    UpdateTopics(Vec<Topic>),
    UpdateSessions(Vec<Session>),
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
    }
}
