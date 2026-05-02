use chrono::{DateTime, Days, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Padding, Paragraph, Row, Table},
};

use crate::state::{Model, StatsFilter, TopicsPane};

pub fn render_topics(frame: &mut Frame, area: Rect, model: &mut Model) {
    let main_layout = Layout::horizontal([Constraint::Fill(5), Constraint::Fill(2)]);

    let [topics, filters] = area.layout(&main_layout);

    let filter_layout = Layout::vertical([Constraint::Fill(1), Constraint::Fill(3)]);

    let [stats, categories] = filters.layout(&filter_layout);

    render_stats(frame, stats, model);
    render_categories(frame, categories, model);
    render_list(frame, topics, model);
}

pub fn render_stats(frame: &mut Frame, area: Rect, model: &mut Model) {
    let block_style = match model.selected_topics_pane {
        TopicsPane::Stats => Style::new().fg(Color::Blue),
        _ => Style::new().fg(Color::Reset),
    };

    let block = Block::bordered()
        .padding(Padding::horizontal(1))
        .title("Stats")
        .border_style(block_style);

    let [total_l, overdue_l, last_7_days_l, mastered_l, struggling_l] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .areas(block.inner(area));

    let seven_days_ago = seven_days_ago();

    let topics_overdue = &model
        .topics
        .iter()
        .filter(|t| t.is_overdue(Utc::now()))
        .collect::<Vec<_>>();

    let last_7_days_topics = &model
        .topics
        .iter()
        .filter(|t| t.is_between(seven_days_ago, Utc::now()))
        .collect::<Vec<_>>();

    let mastered_topics = &model
        .topics
        .iter()
        .filter(|t| t.mastered())
        .collect::<Vec<_>>();

    let struggled_topics = &model
        .topics
        .iter()
        .filter(|t| t.struggled())
        .collect::<Vec<_>>();

    frame.render_widget(block, area);

    let stats = vec![
        (
            total_l,
            StatsFilter::Total,
            "Total",
            model.topics.len(),
            Color::Reset,
        ),
        (
            overdue_l,
            StatsFilter::Overdue,
            "Overdue",
            topics_overdue.len(),
            Color::Red,
        ),
        (
            last_7_days_l,
            StatsFilter::Last7Days,
            "Last 7 days",
            last_7_days_topics.len(),
            Color::Reset,
        ),
        (
            mastered_l,
            StatsFilter::Mastered,
            "Mastered",
            mastered_topics.len(),
            Color::Green,
        ),
        (
            struggling_l,
            StatsFilter::Struggling,
            "Struggling",
            struggled_topics.len(),
            Color::Red,
        ),
    ];

    for (area, stat, label, count, color) in stats.iter() {
        let is_selected = &model.selected_stats_filter == stat;

        let style = if is_selected {
            Style::new().bg(Color::DarkGray).fg(*color)
        } else {
            Style::new().fg(*color)
        };

        let p = Paragraph::new(format!("{} {}", label, count)).style(style);
        frame.render_widget(p, *area);
    }
}

pub fn render_categories(frame: &mut Frame, area: Rect, model: &mut Model) {
    let rows = model.categories.iter().map(|c| {
        let topics = model
            .topics
            .iter()
            .filter(|t| {
                t.categories
                    .0
                    .iter()
                    .any(|category| category.name == c.name)
            })
            .collect::<Vec<_>>();

        let mastered = topics.iter().filter(|t| t.mastered()).collect::<Vec<_>>();
        let struggled = topics.iter().filter(|t| t.struggled()).collect::<Vec<_>>();
        let learning = topics.iter().filter(|t| t.learning()).collect::<Vec<_>>();

        let line = Line::from(vec![
            Span::raw(format!("{}   ", &c.name)),
            Span::styled("█".repeat(mastered.len()), Style::new().fg(Color::Green)),
            Span::styled("▓".repeat(learning.len()), Style::new().fg(Color::Yellow)),
            Span::styled("░".repeat(struggled.len()), Style::new().fg(Color::Red)),
            Span::raw(format!("   {}", topics.len().to_string())),
        ]);

        return Row::new([Cell::from(line)]);
    });

    let table = Table::new(rows, [Constraint::Percentage(100)])
        .row_highlight_style(Style::new().bg(Color::DarkGray));

    let block_style = match model.selected_topics_pane {
        TopicsPane::Categories => Style::new().fg(Color::Blue),
        _ => Style::new().fg(Color::Reset),
    };

    let block = Block::bordered()
        .padding(Padding::horizontal(1))
        .title("Categories")
        .border_style(block_style);

    frame.render_stateful_widget(table, block.inner(area), &mut model.category_state);
    frame.render_widget(block, area);
}

pub fn render_list(frame: &mut Frame, area: Rect, model: &mut Model) {
    let seven_days_ago = seven_days_ago();
    let now = Utc::now();

    let topics_filtered = model.topics.iter().filter(|t| {
        match (
            &model.selected_stats_filter,
            model.category_state.selected(),
        ) {
            (StatsFilter::Total, None) => true,
            (StatsFilter::Total, Some(category)) => t.has_category(&model.categories[category]),
            (StatsFilter::Overdue, None) => t.is_overdue(now),
            (StatsFilter::Overdue, Some(category)) => {
                t.is_overdue(now) && t.has_category(&model.categories[category])
            }
            (StatsFilter::Last7Days, None) => t.is_between(seven_days_ago, now),
            (StatsFilter::Last7Days, Some(category)) => {
                t.is_between(seven_days_ago, now) && t.has_category(&model.categories[category])
            }
            (StatsFilter::Mastered, None) => t.mastered(),
            (StatsFilter::Mastered, Some(category)) => {
                t.mastered() && t.has_category(&model.categories[category])
            }
            (StatsFilter::Struggling, None) => t.struggled(),
            (StatsFilter::Struggling, Some(category)) => {
                t.struggled() && t.has_category(&model.categories[category])
            }
        }
    });

    let rows = topics_filtered.map(|t| {
        let next_review = t.next_review().format("%b %e %T %Y").to_string();

        let next_review_formatted = if t.is_overdue(now) {
            format!("{} (overdue)", next_review)
        } else {
            next_review
        };
        Row::new([
            Cell::from(t.name.as_str()),
            Cell::from(t.ease_factor.to_string()),
            Cell::from(t.repetitions.to_string()),
            Cell::from(t.reviewed_at.format("%b %e %T %Y").to_string()),
            Cell::from(next_review_formatted),
        ])
    });

    let header = Row::new(vec![
        Cell::from("Topic"),
        Cell::from("Ease factor"),
        Cell::from("Repetitions"),
        Cell::from("Last review"),
        Cell::from("Next review"),
    ])
    .style(Style::new().bold())
    .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(9),
            Constraint::Percentage(9),
            Constraint::Percentage(18),
            Constraint::Percentage(24),
        ],
    )
    .row_highlight_style(Style::new().bg(Color::DarkGray))
    .header(header);

    frame.render_stateful_widget(table, area, &mut model.topics_state);
}

pub fn seven_days_ago() -> DateTime<Utc> {
    Utc::now().checked_sub_days(Days::new(7)).unwrap()
}
