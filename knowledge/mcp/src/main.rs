use std::{env::Args, error::Error};

use learning::sqlite::{
    sqlite_session_storage::SqliteSessionStorage, sqlite_topic_storage::SqliteTopicStorage,
};
use rmcp::serve_server;

use crate::tool_service::ToolService;
mod create_session;
mod deserialisers;
mod get_topics;
mod list_all_topics;
mod review_topic;
mod set_topic_categories;
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

    let topic_storage = SqliteTopicStorage::init()?;
    let session_storage = SqliteSessionStorage::init()?;

    serve_server(
        ToolService::new(Box::new(topic_storage), Box::new(session_storage)),
        io,
    )
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
