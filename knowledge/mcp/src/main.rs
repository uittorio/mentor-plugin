use std::error::Error;

use rmcp::serve_server;

use crate::tool_service::ToolService;
mod deserialisers;
mod get_topics;
mod review_topic;
mod tool_service;
mod topic_candidates;
mod topic_depth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let io = (tokio::io::stdin(), tokio::io::stdout());

    let storage = topic::sqlite_topic_storage::SqliteTopicStorage::init()?;

    serve_server(ToolService::new(Box::new(storage)), io)
        .await?
        .waiting()
        .await?;
    Ok(())
}
