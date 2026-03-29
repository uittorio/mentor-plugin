use std::error::Error;

mod sm2;
mod sqlite_topic_storage;
mod tools;
mod topic;
mod trigram_similarity;
use rmcp::serve_server;

use crate::{sqlite_topic_storage::SqliteTopicStorage, tools::tool_service::ToolService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let io = (tokio::io::stdin(), tokio::io::stdout());

    let storage = SqliteTopicStorage::init()?;
    serve_server(ToolService::new(Box::new(storage)), io)
        .await?
        .waiting()
        .await?;
    Ok(())
}
