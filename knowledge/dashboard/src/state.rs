use std::collections::HashSet;

use learning::{category::Category, session::Session, topic::Topic};
use ratatui::widgets::TableState;

#[derive(Copy, Clone)]
pub enum View {
    Topics,
    Sessions,
}

#[derive(Copy, Clone)]
pub enum SessionsPane {
    List,
    SessionMd,
}

pub struct Model {
    pub topics: Vec<Topic>,
    pub categories: Vec<Category>,
    pub sessions: Vec<Session>,
    pub selected_view: View,
    pub topics_state: TableState,
    pub session_state: TableState,
    pub selected_session_pane: SessionsPane,
    pub session_md_scroll: u16,
    pub selected_topics_pane: TopicsPane,
    pub selected_stats_filter: StatsFilter,
    pub category_state: TableState,
}

#[derive(PartialEq)]
pub enum StatsFilter {
    Total,
    Overdue,
    Last7Days,
    Mastered,
    Struggling,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TopicsPane {
    List,
    Stats,
    Categories,
}

impl Model {
    pub fn new() -> Self {
        Model {
            selected_view: View::Topics,
            topics: vec![],
            categories: vec![],
            sessions: vec![],
            topics_state: TableState::default(),
            session_state: TableState::default(),
            selected_session_pane: SessionsPane::List,
            session_md_scroll: 0,
            selected_topics_pane: TopicsPane::List,
            selected_stats_filter: StatsFilter::Total,
            category_state: TableState::default(),
        }
    }
}

pub enum Message {
    ShowTopicView,
    ShowSessionView,
    UpdateTopics(Vec<Topic>),
    UpdateSessions(Vec<Session>),
    NavigateUp,
    NavigateDown,
    NextSessionPane,
    PrevPane,
    ResetFilters,
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
        Message::ShowTopicView => model.selected_view = View::Topics,
        Message::ShowSessionView => model.selected_view = View::Sessions,
        Message::NextSessionPane | Message::PrevPane => {
            match (
                model.selected_view,
                model.selected_session_pane,
                model.selected_topics_pane,
            ) {
                (View::Topics, _, TopicsPane::Categories) => {
                    model.selected_topics_pane = TopicsPane::List
                }
                (View::Topics, _, TopicsPane::Stats) => {
                    model.selected_topics_pane = TopicsPane::Categories
                }
                (View::Topics, _, TopicsPane::List) => {
                    model.selected_topics_pane = TopicsPane::Stats
                }
                (View::Sessions, SessionsPane::List, _) => {
                    model.selected_session_pane = SessionsPane::SessionMd
                }
                (View::Sessions, SessionsPane::SessionMd, _) => {
                    model.selected_session_pane = SessionsPane::List
                }
            }
        }
        Message::UpdateTopics(topics) => {
            let mut categories: Vec<Category> = topics
                .iter()
                .flat_map(|t| t.categories.0.iter().map(|c| c.name.clone()))
                .collect::<HashSet<String>>()
                .into_iter()
                .map(|name| Category { name })
                .collect();

            categories.sort_by(|a, b| a.name.cmp(&b.name));

            model.topics = topics;
            model.categories = categories;
        }
        Message::UpdateSessions(sessions) => {
            model.sessions = sessions;
        }
        Message::NavigateUp => match model.selected_view {
            View::Topics => match model.selected_topics_pane {
                TopicsPane::List => update_selected_table_up(&mut model.topics_state),
                TopicsPane::Stats => {
                    model.selected_stats_filter = match model.selected_stats_filter {
                        StatsFilter::Total => StatsFilter::Total,
                        StatsFilter::Overdue => StatsFilter::Total,
                        StatsFilter::Last7Days => StatsFilter::Overdue,
                        StatsFilter::Mastered => StatsFilter::Last7Days,
                        StatsFilter::Struggling => StatsFilter::Mastered,
                    }
                }
                TopicsPane::Categories => update_selected_table_up(&mut model.category_state),
            },
            View::Sessions => match model.selected_session_pane {
                SessionsPane::List => update_selected_table_up(&mut model.session_state),
                SessionsPane::SessionMd => model.session_md_scroll = model.session_md_scroll + 1,
            },
        },
        Message::NavigateDown => match model.selected_view {
            View::Topics => match model.selected_topics_pane {
                TopicsPane::List => {
                    update_selected_table_down(&mut model.topics_state, model.topics.len())
                }
                TopicsPane::Stats => {
                    model.selected_stats_filter = match model.selected_stats_filter {
                        StatsFilter::Total => StatsFilter::Overdue,
                        StatsFilter::Overdue => StatsFilter::Last7Days,
                        StatsFilter::Last7Days => StatsFilter::Mastered,
                        StatsFilter::Mastered => StatsFilter::Struggling,
                        StatsFilter::Struggling => StatsFilter::Struggling,
                    };
                }
                TopicsPane::Categories => {
                    update_selected_table_down(&mut model.category_state, model.categories.len())
                }
            },
            View::Sessions => match model.selected_session_pane {
                SessionsPane::List => {
                    update_selected_table_down(&mut model.session_state, model.sessions.len())
                }
                SessionsPane::SessionMd => {
                    model.session_md_scroll = model.session_md_scroll.saturating_sub(1)
                }
            },
        },
        Message::ResetFilters => model.category_state.select(None),
    }
}
