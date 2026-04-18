use std::collections::BTreeMap;

use chrono::DateTime;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, Cell, Row, Table},
};

use crate::state::Model;
use learning::topic::{QuestionDepth, Topic};

struct CategorySummary<'a> {
    name: String,
    topics: Vec<&'a Topic>,
}

fn build_summaries(topics: &[Topic]) -> Vec<CategorySummary<'_>> {
    let mut map: BTreeMap<String, Vec<&Topic>> = BTreeMap::new();

    for topic in topics {
        if topic.categories.is_empty() {
            map.entry("Uncategorized".to_string()).or_default().push(topic);
        } else {
            for cat in &topic.categories {
                map.entry(cat.clone()).or_default().push(topic);
            }
        }
    }

    // Sort: named categories first (alphabetical), Uncategorized last
    let mut summaries: Vec<CategorySummary> = map
        .into_iter()
        .map(|(name, topics)| CategorySummary { name, topics })
        .collect();

    summaries.sort_by(|a, b| {
        match (a.name.as_str(), b.name.as_str()) {
            ("Uncategorized", _) => std::cmp::Ordering::Greater,
            (_, "Uncategorized") => std::cmp::Ordering::Less,
            _ => a.name.cmp(&b.name),
        }
    });

    summaries
}

pub fn render_categories(frame: &mut Frame, area: Rect, model: &mut Model) {
    let now_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|t| t.as_secs())
        .unwrap_or(0);

    let summaries = build_summaries(&model.topics);

    let [left, right] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .areas(area);

    // --- Left pane: category summary table ---
    let cat_rows: Vec<Row> = summaries.iter().map(|s| {
        let overdue = s.topics.iter().filter(|t| {
            let next = t.reviewed_at + (t.interval as u64 * 86400);
            next < now_epoch
        }).count();
        let mastered = s.topics.iter().filter(|t| {
            matches!(t.question_depth(), QuestionDepth::Skip)
        }).count();

        let overdue_cell = if overdue > 0 {
            Cell::from(overdue.to_string()).style(Style::new().fg(Color::Red))
        } else {
            Cell::from("0")
        };
        let mastered_cell = if mastered > 0 {
            Cell::from(mastered.to_string()).style(Style::new().fg(Color::Green))
        } else {
            Cell::from("0")
        };

        Row::new([
            Cell::from(s.name.clone()),
            Cell::from(s.topics.len().to_string()),
            overdue_cell,
            mastered_cell,
        ])
    }).collect();

    let cat_header = Row::new(vec![
        Cell::from("Category"),
        Cell::from("Topics"),
        Cell::from("Overdue"),
        Cell::from("Mastered"),
    ])
    .style(Style::new().bold())
    .bottom_margin(1);

    let cat_table = Table::new(
        cat_rows,
        [
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .block(Block::bordered().title(Span::from(" Categories ").bold()))
    .row_highlight_style(Style::new().bg(Color::DarkGray))
    .header(cat_header);

    frame.render_stateful_widget(cat_table, left, &mut model.categories_state);

    // --- Right pane: topics for selected category ---
    let selected_topics: Vec<&Topic> = match model.categories_state.selected() {
        Some(idx) => summaries.get(idx).map(|s| s.topics.clone()).unwrap_or_default(),
        None => model.topics.iter().collect(),
    };

    let topic_rows: Vec<Row> = selected_topics.iter().map(|t| {
        let next_review = t.reviewed_at + (t.interval as u64 * 86400);
        let is_overdue = next_review < now_epoch;
        let next_str = {
            let formatted = DateTime::from_timestamp_secs(next_review as i64)
                .unwrap()
                .format("%b %e %Y")
                .to_string();
            if is_overdue { format!("{} (overdue)", formatted) } else { formatted }
        };

        let row_style = match (is_overdue, t.question_depth()) {
            (true, _) => Style::new().fg(Color::Red),
            (_, QuestionDepth::Skip) => Style::new().fg(Color::Green),
            (_, QuestionDepth::Light) => Style::new().fg(Color::Yellow),
            _ => Style::new(),
        };

        Row::new([
            Cell::from(t.name.clone()),
            Cell::from(format!("{:.2}", t.ease_factor)),
            Cell::from(t.repetitions.to_string()),
            Cell::from(next_str),
        ])
        .style(row_style)
    }).collect();

    let topic_header = Row::new(vec![
        Cell::from("Topic"),
        Cell::from("Ease"),
        Cell::from("Reps"),
        Cell::from("Next review"),
    ])
    .style(Style::new().bold())
    .bottom_margin(1);

    let selected_cat_name = model
        .categories_state
        .selected()
        .and_then(|i| summaries.get(i))
        .map(|s| format!(" {} ", s.name))
        .unwrap_or_else(|| " All topics ".to_string());

    let topic_table = Table::new(
        topic_rows,
        [
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Percentage(30),
        ],
    )
    .block(Block::bordered().title(Span::from(selected_cat_name).bold()))
    .header(topic_header);

    frame.render_widget(topic_table, right);
}
