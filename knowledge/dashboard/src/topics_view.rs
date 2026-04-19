use chrono::DateTime;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Paragraph, Row, Table},
};

use crate::state::{
    Model, StatFilter, Zone, STAT_COUNT, STAT_FILTERS, STAT_LABELS, filtered_topics,
    sorted_categories,
};
use learning::topic::{QuestionDepth, Topic};

// Re-exported for state.rs navigation helpers
pub fn update_selected_table_up(state: &mut ratatui::widgets::TableState) {
    match state.selected() {
        Some(s) => state.select(Some(if s == 0 { 0 } else { s - 1 })),
        None => state.select(Some(0)),
    }
}

pub fn update_selected_table_down(state: &mut ratatui::widgets::TableState, len: usize) {
    if len == 0 {
        return;
    }
    match state.selected() {
        Some(s) => state.select(Some(if s + 1 < len { s + 1 } else { s })),
        None => state.select(Some(0)),
    }
}

pub fn epoch_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|t| t.as_secs())
        .unwrap_or(0)
}

pub fn render_topics_view(frame: &mut Frame, area: Rect, model: &mut Model) {
    let now = epoch_now();

    let categories = sorted_categories(&model.topics);
    // +2 for border, cap between 4 and 12 visible rows
    let chart_height = ((categories.len() as u16) + 2).clamp(4, 12);

    let [stats_area, chart_area, list_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(chart_height),
        Constraint::Fill(1),
    ])
    .areas(area);

    render_stats_bar(frame, stats_area, model, now);
    render_categories_chart(frame, chart_area, model, &categories, now);
    render_topics_list(frame, list_area, model, now);
}

// --- Stats bar ---

fn render_stats_bar(frame: &mut Frame, area: Rect, model: &Model, now: u64) {
    let is_focused = model.focused_zone == Some(Zone::Stats);

    let overdue = model
        .topics
        .iter()
        .filter(|t| t.reviewed_at + (t.interval as u64 * 86400) < now)
        .count();
    let due_week = model
        .topics
        .iter()
        .filter(|t| {
            let next = t.reviewed_at + (t.interval as u64 * 86400);
            next >= now && next < now + 7 * 86400
        })
        .count();
    let mastered = model
        .topics
        .iter()
        .filter(|t| matches!(t.question_depth(), QuestionDepth::Skip))
        .count();
    let struggling = model
        .topics
        .iter()
        .filter(|t| matches!(t.question_depth(), QuestionDepth::Full))
        .count();
    let values = [overdue, due_week, mastered, struggling];
    let value_colors = [Color::Red, Color::Yellow, Color::Green, Color::Red];

    // 5 equal columns: Total + 4 filterable stats
    let [total_area, s0, s1, s2, s3] = Layout::horizontal([
        Constraint::Ratio(1, 5),
        Constraint::Ratio(1, 5),
        Constraint::Ratio(1, 5),
        Constraint::Ratio(1, 5),
        Constraint::Ratio(1, 5),
    ])
    .areas(area);
    let stat_areas = [s0, s1, s2, s3];

    let zone_border = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Plain
    };

    frame.render_widget(
        Paragraph::new(format!("Total: {}", model.topics.len()))
            .centered()
            .block(Block::bordered().border_type(zone_border)),
        total_area,
    );

    for i in 0..STAT_COUNT {
        let is_cursor = is_focused && model.stats_cursor == i;
        let is_active = model.active_stat_filters.contains(&STAT_FILTERS[i]);
        let col = value_colors[i];

        let (bg, text_style, border_style) = match (is_cursor, is_active) {
            (true, true) => (
                Color::DarkGray,
                Style::new().fg(col).bold(),
                Style::new().fg(Color::Cyan),
            ),
            (true, false) => (
                Color::DarkGray,
                Style::new(),
                Style::new().fg(Color::Cyan),
            ),
            (false, true) => (
                Color::Reset,
                Style::new().fg(col).bold(),
                Style::new().fg(col),
            ),
            (false, false) => (Color::Reset, Style::new(), Style::new()),
        };

        frame.render_widget(
            Paragraph::new(format!("{}: {}", STAT_LABELS[i], values[i]))
                .centered()
                .style(text_style.bg(bg))
                .block(
                    Block::bordered()
                        .border_type(zone_border)
                        .border_style(border_style),
                ),
            stat_areas[i],
        );
    }
}

// --- Categories chart ---

fn render_categories_chart(
    frame: &mut Frame,
    area: Rect,
    model: &Model,
    categories: &[String],
    _now: u64,
) {
    let is_focused = model.focused_zone == Some(Zone::Categories);

    let has_active = !model.active_category_filters.is_empty();
    let title = if has_active {
        let names: Vec<&str> = model
            .active_category_filters
            .iter()
            .map(|s| s.as_str())
            .collect();
        format!(" Categories · filtered: {} ", names.join(", "))
    } else {
        " Categories (2 to focus, j/k navigate, Space select) ".to_string()
    };

    let block = Block::bordered()
        .title(title)
        .border_type(if is_focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .border_style(if is_focused {
            Style::new().fg(Color::Cyan)
        } else {
            Style::default()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if categories.is_empty() {
        return;
    }

    let max_count = categories
        .iter()
        .map(|c| topics_for_category(&model.topics, c).len())
        .max()
        .unwrap_or(1)
        .max(1);

    // Fixed widths: name col + bar + count
    let name_col: usize = 20;
    let count_col: usize = 5;
    let bar_max: usize = (inner.width as usize).saturating_sub(name_col + count_col + 1);

    // Scroll to keep cursor visible
    let visible = inner.height as usize;
    let scroll = if model.categories_cursor >= visible {
        model.categories_cursor - visible + 1
    } else {
        0
    };

    for (display_idx, cat) in categories.iter().skip(scroll).take(visible).enumerate() {
        let real_idx = display_idx + scroll;
        let is_cursor = is_focused && model.categories_cursor == real_idx;
        let is_active = model.active_category_filters.contains(cat.as_str());

        let cat_topics = topics_for_category(&model.topics, cat);
        let total = cat_topics.len();
        let mastered = cat_topics
            .iter()
            .filter(|t| matches!(t.question_depth(), QuestionDepth::Skip))
            .count();
        let learning = cat_topics
            .iter()
            .filter(|t| matches!(t.question_depth(), QuestionDepth::Light))
            .count();
        let _struggling = total - mastered - learning;

        // Scale bar
        let bar_total = (total * bar_max) / max_count;
        let bar_mastered = if total > 0 { (mastered * bar_total) / total } else { 0 };
        let bar_learning = if total > 0 { (learning * bar_total) / total } else { 0 };
        let bar_struggling = bar_total - bar_mastered - bar_learning;

        let bg = if is_cursor { Color::DarkGray } else { Color::Reset };
        let name_style = if is_active {
            Style::new().fg(Color::Cyan).bold().bg(bg)
        } else {
            Style::new().bg(bg)
        };

        let name_display = truncate_pad(cat, name_col);

        let line = Line::from(vec![
            Span::styled(name_display, name_style),
            Span::styled(
                "█".repeat(bar_mastered),
                Style::new().fg(Color::Green).bg(bg),
            ),
            Span::styled(
                "▓".repeat(bar_learning),
                Style::new().fg(Color::Yellow).bg(bg),
            ),
            Span::styled(
                "░".repeat(bar_struggling),
                Style::new().fg(Color::Red).bg(bg),
            ),
            Span::styled(format!(" {:>3}", total), Style::new().bg(bg)),
        ]);

        let row_rect = Rect::new(inner.x, inner.y + display_idx as u16, inner.width, 1);
        frame.render_widget(Paragraph::new(line), row_rect);
    }
}

// --- Filtered topics list ---

fn render_topics_list(frame: &mut Frame, area: Rect, model: &mut Model, now: u64) {
    let is_focused =
        model.focused_zone == Some(Zone::Topics) || model.focused_zone.is_none();

    let topics = filtered_topics(
        &model.topics,
        &model.active_stat_filters,
        &model.active_category_filters,
        now,
    );

    let title = filter_title(model, topics.len());

    let block = Block::bordered()
        .title(title)
        .border_type(if is_focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .border_style(if is_focused {
            Style::new().fg(Color::Cyan)
        } else {
            Style::default()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows: Vec<Row> = topics
        .iter()
        .map(|t| {
            let next = t.reviewed_at + (t.interval as u64 * 86400);
            let is_over = next < now;
            let next_str = {
                let fmt = DateTime::from_timestamp_secs(next as i64)
                    .unwrap()
                    .format("%b %e %Y")
                    .to_string();
                if is_over {
                    format!("{} (overdue)", fmt)
                } else {
                    fmt
                }
            };
            let cats = if t.categories.is_empty() {
                "-".to_string()
            } else {
                t.categories.join(", ")
            };
            let row_style = match (is_over, t.question_depth()) {
                (true, _) => Style::new().fg(Color::Red),
                (_, QuestionDepth::Skip) => Style::new().fg(Color::Green),
                (_, QuestionDepth::Light) => Style::new().fg(Color::Yellow),
                _ => Style::new(),
            };
            Row::new([
                Cell::from(t.name.clone()),
                Cell::from(cats),
                Cell::from(format!("{:.1}", t.ease_factor)),
                Cell::from(t.repetitions.to_string()),
                Cell::from(next_str),
            ])
            .style(row_style)
        })
        .collect();

    let header = Row::new([
        Cell::from("Topic"),
        Cell::from("Categories"),
        Cell::from("Ease"),
        Cell::from("Reps"),
        Cell::from("Next review"),
    ])
    .style(Style::new().bold())
    .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(27),
            Constraint::Percentage(22),
            Constraint::Percentage(6),
            Constraint::Percentage(5),
            Constraint::Percentage(40),
        ],
    )
    .header(header)
    .row_highlight_style(Style::new().bg(Color::DarkGray));

    frame.render_stateful_widget(table, inner, &mut model.topics_state);
}

// --- Helpers ---

fn topics_for_category<'a>(topics: &'a [Topic], category: &str) -> Vec<&'a Topic> {
    if category == "Uncategorized" {
        topics.iter().filter(|t| t.categories.is_empty()).collect()
    } else {
        topics
            .iter()
            .filter(|t| t.categories.iter().any(|c| c == category))
            .collect()
    }
}

fn filter_title(model: &Model, count: usize) -> String {
    let total = model.topics.len();
    let mut parts: Vec<String> = model
        .active_stat_filters
        .iter()
        .map(|f| match f {
            StatFilter::Overdue => "Overdue".to_string(),
            StatFilter::DueThisWeek => "Due this week".to_string(),
            StatFilter::Mastered => "Mastered".to_string(),
            StatFilter::Struggling => "Struggling".to_string(),
        })
        .chain(model.active_category_filters.iter().cloned())
        .collect();
    parts.sort();

    if parts.is_empty() {
        format!(" Topics ({}) ", total)
    } else {
        format!(" Topics ({}/{}) · {} ", count, total, parts.join(", "))
    }
}

fn truncate_pad(s: &str, width: usize) -> String {
    if s.len() >= width {
        format!("{} ", &s[..width - 1])
    } else {
        format!("{:<width$}", s, width = width)
    }
}
