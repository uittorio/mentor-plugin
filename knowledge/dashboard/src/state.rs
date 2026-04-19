use std::collections::HashSet;

use learning::{session::Session, topic::Topic};
use ratatui::widgets::TableState;

use crate::topics_view::{update_selected_table_down, update_selected_table_up};

#[derive(Copy, Clone, PartialEq)]
pub enum View {
    Topics,
    Sessions,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Zone {
    Stats,
    Categories,
    Topics,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum StatFilter {
    Overdue,
    DueThisWeek,
    Mastered,
    Struggling,
}

pub const STAT_COUNT: usize = 4;
pub const STAT_FILTERS: [StatFilter; STAT_COUNT] = [
    StatFilter::Overdue,
    StatFilter::DueThisWeek,
    StatFilter::Mastered,
    StatFilter::Struggling,
];
pub const STAT_LABELS: [&str; STAT_COUNT] = ["Overdue", "Due this week", "Mastered", "Struggling"];

#[derive(Copy, Clone)]
pub enum Pane {
    Sessions,
    SessionMd,
}

pub struct Model {
    pub selected_view: View,
    pub topics: Vec<Topic>,
    pub sessions: Vec<Session>,

    // Zone focus (Topics view)
    pub focused_zone: Option<Zone>,

    // Stats zone
    pub stats_cursor: usize,
    pub active_stat_filters: HashSet<StatFilter>,

    // Categories zone
    pub categories_cursor: usize,
    pub active_category_filters: HashSet<String>,

    // Topics list
    pub topics_state: TableState,

    // Sessions view
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
            focused_zone: None,
            stats_cursor: 0,
            active_stat_filters: HashSet::new(),
            categories_cursor: 0,
            active_category_filters: HashSet::new(),
            topics_state: TableState::default(),
            session_state: TableState::default(),
            focused_pane: Pane::Sessions,
            session_md_scroll: 0,
        }
    }
}

pub enum Message {
    ShowTopicView,
    ShowSessionView,
    UpdateTopics(Vec<Topic>),
    UpdateSessions(Vec<Session>),
    FocusZone(Zone),
    ExitZone,
    ToggleFilter,
    ResetFilters,
    NextPane,
    PrevPane,
}

pub fn update(model: &mut Model, msg: Message) {
    match msg {
        Message::ShowTopicView => model.selected_view = View::Topics,
        Message::ShowSessionView => model.selected_view = View::Sessions,
        Message::UpdateTopics(topics) => model.topics = topics,
        Message::UpdateSessions(sessions) => model.sessions = sessions,

        Message::FocusZone(zone) => {
            if model.focused_zone == Some(zone) {
                model.focused_zone = None;
            } else {
                model.focused_zone = Some(zone);
            }
        }
        Message::ExitZone => model.focused_zone = None,

        Message::ResetFilters => {
            model.active_stat_filters.clear();
            model.active_category_filters.clear();
        }

        Message::ToggleFilter => match model.focused_zone {
            Some(Zone::Stats) => {
                let filter = STAT_FILTERS[model.stats_cursor];
                if model.active_stat_filters.contains(&filter) {
                    model.active_stat_filters.remove(&filter);
                } else {
                    model.active_stat_filters.insert(filter);
                }
            }
            Some(Zone::Categories) => {
                let cats = sorted_categories(&model.topics);
                if let Some(cat) = cats.into_iter().nth(model.categories_cursor) {
                    if model.active_category_filters.contains(&cat) {
                        model.active_category_filters.remove(&cat);
                    } else {
                        model.active_category_filters.insert(cat);
                    }
                }
            }
            _ => {}
        },

        Message::NextPane | Message::PrevPane => {
            if let View::Sessions = model.selected_view {
                model.focused_pane = match model.focused_pane {
                    Pane::Sessions => Pane::SessionMd,
                    Pane::SessionMd => Pane::Sessions,
                };
            }
        }
    }
}

// --- Navigation (called directly from main, needs context) ---

pub fn navigate_down(model: &mut Model, now: u64) {
    match (model.selected_view, model.focused_zone) {
        (View::Topics, Some(Zone::Categories)) => {
            let count = sorted_categories(&model.topics).len();
            if model.categories_cursor + 1 < count {
                model.categories_cursor += 1;
            }
        }
        (View::Topics, Some(Zone::Stats)) => {
            if model.stats_cursor + 1 < STAT_COUNT {
                model.stats_cursor += 1;
            }
        }
        (View::Topics, _) => {
            let count = filtered_topics(&model.topics, &model.active_stat_filters, &model.active_category_filters, now).len();
            update_selected_table_down(&mut model.topics_state, count);
        }
        (View::Sessions, _) => match model.focused_pane {
            Pane::Sessions => {
                update_selected_table_down(&mut model.session_state, model.sessions.len())
            }
            Pane::SessionMd => {
                model.session_md_scroll = model.session_md_scroll.saturating_sub(1)
            }
        },
    }
}

pub fn navigate_up(model: &mut Model) {
    match (model.selected_view, model.focused_zone) {
        (View::Topics, Some(Zone::Categories)) => {
            if model.categories_cursor > 0 {
                model.categories_cursor -= 1;
            }
        }
        (View::Topics, Some(Zone::Stats)) => {
            if model.stats_cursor > 0 {
                model.stats_cursor -= 1;
            }
        }
        (View::Topics, _) => {
            update_selected_table_up(&mut model.topics_state);
        }
        (View::Sessions, _) => match model.focused_pane {
            Pane::Sessions => update_selected_table_up(&mut model.session_state),
            Pane::SessionMd => model.session_md_scroll += 1,
        },
    }
}

pub fn navigate_left(model: &mut Model) {
    if let Some(Zone::Stats) = model.focused_zone {
        if model.stats_cursor > 0 {
            model.stats_cursor -= 1;
        }
    }
}

pub fn navigate_right(model: &mut Model) {
    if let Some(Zone::Stats) = model.focused_zone {
        if model.stats_cursor + 1 < STAT_COUNT {
            model.stats_cursor += 1;
        }
    }
}

// --- Helpers used by view and state ---

pub fn sorted_categories(topics: &[Topic]) -> Vec<String> {
    let mut named: Vec<String> = {
        let set: std::collections::BTreeSet<String> = topics
            .iter()
            .flat_map(|t| t.categories.iter().cloned())
            .collect();
        set.into_iter().collect()
    };
    if topics.iter().any(|t| t.categories.is_empty()) {
        named.push("Uncategorized".to_string());
    }
    named
}

pub fn filtered_topics<'a>(
    topics: &'a [Topic],
    stat_filters: &HashSet<StatFilter>,
    cat_filters: &HashSet<String>,
    now: u64,
) -> Vec<&'a Topic> {
    use learning::topic::QuestionDepth;

    topics
        .iter()
        .filter(|t| {
            let matches_stat = if stat_filters.is_empty() {
                true
            } else {
                stat_filters.iter().any(|f| match f {
                    StatFilter::Overdue => {
                        let next = t.reviewed_at + (t.interval as u64 * 86400);
                        next < now
                    }
                    StatFilter::DueThisWeek => {
                        let next = t.reviewed_at + (t.interval as u64 * 86400);
                        next >= now && next < now + 7 * 86400
                    }
                    StatFilter::Mastered => matches!(t.question_depth(), QuestionDepth::Skip),
                    StatFilter::Struggling => matches!(t.question_depth(), QuestionDepth::Full),
                })
            };

            let matches_cat = if cat_filters.is_empty() {
                true
            } else {
                cat_filters.iter().any(|c| {
                    if c == "Uncategorized" {
                        t.categories.is_empty()
                    } else {
                        t.categories.contains(c)
                    }
                })
            };

            matches_stat && matches_cat
        })
        .collect()
}
