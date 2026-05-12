use std::{fs, path::Path, process::Command};

use learning::file_storage::file_storage_folder;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReviewTopicCommand {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Deserialize)]
pub struct DashboardConfig {
    pub review_topic_commands: Vec<ReviewTopicCommand>,
}

impl ReviewTopicCommand {
    pub fn start(&self, topic_name: &str) -> color_eyre::Result<()> {
        let replacement = format!(
            "Activate /mentor+ skill and then start a session to review the topic named {}",
            topic_name
        );

        let processed_args: Vec<String> = self
            .args
            .iter()
            .map(|arg| arg.replace("$MentorPrompt", &replacement))
            .collect();

        let mut args = processed_args.iter();
        let first_arg = args.next().unwrap();

        Command::new(first_arg).args(args).spawn()?;
        Ok(())
    }
}

pub fn config() -> Option<DashboardConfig> {
    let folder = file_storage_folder();
    let config_path = Path::new(&folder)
        .join("dashboard.toml")
        .to_str()
        .unwrap()
        .to_string();

    fs::read_to_string(config_path)
        .ok()
        .and_then(|f| toml::from_str::<DashboardConfig>(&f).ok())
}
