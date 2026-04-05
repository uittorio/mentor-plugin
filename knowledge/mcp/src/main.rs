use std::{env::Args, error::Error};

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
    let mut args = std::env::args();

    if has_version_argument(&mut args) {
        print_version();
        return Ok(());
    }

    let io = (tokio::io::stdin(), tokio::io::stdout());

    let storage = topic::sqlite_topic_storage::SqliteTopicStorage::init()?;

    serve_server(ToolService::new(Box::new(storage)), io)
        .await?
        .waiting()
        .await?;
    Ok(())
}

fn has_version_argument(args: &mut Args) -> bool {
    return args.any(|a| a == "--version" || a == "-v");
}

fn print_version() -> () {
    let version = env!("CARGO_PKG_VERSION");
    println!("{}", version);
}
