use learning::{session::Session, topic::Topic};
use ratatui::widgets::TableState;

#[derive(Copy, Clone, PartialEq)]
pub enum View {
    Topics,
    Categories,
    Sessions,
}

#[derive(Copy, Clone)]
pub enum Pane {
    Sessions,
    SessionMd,
}

pub struct Model {
    pub topics: Vec<Topic>,
    pub sessions: Vec<Session>,
    pub selected_view: View,
    pub topics_state: TableState,
    pub categories_state: TableState,
    pub session_state: TableState,
    pub focused_pane: Pane,
    pub session_md_scroll: u16,
}

impl Model {
    pub fn new() -> Self {
        Model {
            selected_view: View::Topics,
            topics: vec![],
            sessions: vec![],
            topics_state: TableState::default(),
            categories_state: TableState::default(),
            session_state: TableState::default(),
            focused_pane: Pane::Sessions,
            session_md_scroll: 0,
        }
    }
}

pub enum Message {
    ShowTopicView,
    ShowCategoriesView,
    ShowSessionView,
    UpdateTopics(Vec<Topic>),
    UpdateSessions(Vec<Session>),
    NavigateUp,
    NavigateDown,
    NextPane,
    PrevPane,
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
    if list_len == 0 {
        return;
    }
    match table_state.selected() {
        Some(s) => {
            let down = if s != list_len - 1 { s + 1 } else { s };
            table_state.select(Some(down))
        }
        None => table_state.select(Some(0)),
    };
}

pub fn update(model: &mut Model, msg: Message) {
    match msg {
        Message::ShowTopicView => model.selected_view = View::Topics,
        Message::ShowCategoriesView => model.selected_view = View::Categories,
        Message::ShowSessionView => model.selected_view = View::Sessions,
        Message::NextPane | Message::PrevPane => match (model.selected_view, model.focused_pane) {
            (View::Topics, _) | (View::Categories, _) => {}
            (View::Sessions, Pane::Sessions) => model.focused_pane = Pane::SessionMd,
            (View::Sessions, Pane::SessionMd) => model.focused_pane = Pane::Sessions,
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
            View::Categories => {
                update_selected_table_up(&mut model.categories_state);
            }
            View::Sessions => match model.focused_pane {
                Pane::Sessions => update_selected_table_up(&mut model.session_state),
                Pane::SessionMd => model.session_md_scroll = model.session_md_scroll + 1,
            },
        },
        Message::NavigateDown => match model.selected_view {
            View::Topics => {
                update_selected_table_down(&mut model.topics_state, model.topics.len());
            }
            View::Categories => {
                let cat_count = category_count(&model.topics);
                update_selected_table_down(&mut model.categories_state, cat_count);
            }
            View::Sessions => match model.focused_pane {
                Pane::Sessions => {
                    update_selected_table_down(&mut model.session_state, model.sessions.len())
                }
                Pane::SessionMd => {
                    model.session_md_scroll = model.session_md_scroll.saturating_sub(1)
                }
            },
        },
    }
}

pub fn category_count(topics: &[Topic]) -> usize {
    let mut cats: Vec<&str> = topics
        .iter()
        .flat_map(|t| t.categories.iter().map(|c| c.as_str()))
        .collect();
    cats.sort_unstable();
    cats.dedup();
    let named = cats.len();
    let has_uncategorized = topics.iter().any(|t| t.categories.is_empty());
    if has_uncategorized { named + 1 } else { named }
}
