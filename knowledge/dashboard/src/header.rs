use ratatui::{
    Frame,
    layout::{Constraint, HorizontalAlignment, Layout, Offset, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Tabs},
};

use crate::state::{Model, View};

struct KeyConfiguration {
    key: String,
    message: String,
}

pub fn render_header(frame: &mut Frame, area: Rect, model: &mut Model) {
    let mut configuration = vec![KeyConfiguration {
        key: "q".to_string(),
        message: "quit".to_string(),
    }];

    let mut configuration_topic = vec![
        KeyConfiguration {
            key: "s".to_string(),
            message: "sessions".to_string(),
        },
        KeyConfiguration {
            key: "l".to_string(),
            message: "logs".to_string(),
        },
        KeyConfiguration {
            key: "→|←".to_string(),
            message: "switch pane".to_string(),
        },
        KeyConfiguration {
            key: "Esc".to_string(),
            message: "reset category filter".to_string(),
        },
    ];

    if let Some(config) = model.selected_review_topic_command {
        let name = &model.config.as_ref().unwrap().review_topic_commands[config].name;
        configuration_topic.push(KeyConfiguration {
            key: "a".to_string(),
            message: format!("review topic configuration ({})", name),
        });
    };

    let mut configuration_session = vec![
        KeyConfiguration {
            key: "t".to_string(),
            message: "topics".to_string(),
        },
        KeyConfiguration {
            key: "l".to_string(),
            message: "logs".to_string(),
        },
    ];

    let mut configuration_logs = vec![
        KeyConfiguration {
            key: "t".to_string(),
            message: "topics".to_string(),
        },
        KeyConfiguration {
            key: "s".to_string(),
            message: "sessions".to_string(),
        },
    ];

    match model.selected_view {
        View::Topics => configuration.append(&mut configuration_topic),
        View::Sessions => configuration.append(&mut configuration_session),
        View::Logs => configuration.append(&mut configuration_logs),
    };

    let instructions = configuration
        .iter()
        .map(|c| format!("({}) {}", c.key, c.message))
        .collect::<Vec<_>>()
        .join(", ");

    let instructions = Paragraph::new(instructions);
    frame.render_widget(instructions.centered(), area);
    render_tabs(frame, area + Offset::new(0, 2), model);
}

fn render_tabs(frame: &mut Frame, area: Rect, model: &Model) {
    let [_, center, right] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(30),
        Constraint::Fill(1),
    ])
    .areas(area);

    let tabs = Tabs::new(vec!["Topics", "Sessions", "Logs"])
        .style(Color::White)
        .highlight_style(Style::default().magenta().on_black().bold())
        .select(model.selected_view as usize)
        .divider("|")
        .padding(" ", " ");
    frame.render_widget(tabs, center);

    let database = match model.connection_type {
        learning::sql::sql_storage::ConnectionType::Remote => Paragraph::new("database: turso")
            .alignment(HorizontalAlignment::Right)
            .style(Style::new().fg(Color::Green)),
        learning::sql::sql_storage::ConnectionType::Local => Paragraph::new("database: local")
            .alignment(HorizontalAlignment::Right)
            .style(Style::new().fg(Color::DarkGray)),
    };
    frame.render_widget(database, right);
}
