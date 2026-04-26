use std::{env::Args, error::Error, sync::Arc};

use learning::sql::{
    sql_session_storage::SqlSessionStorage, sql_storage::SqlConnection,
    sql_topic_storage::SqlTopicStorage,
};
use rmcp::serve_server;

use crate::tool_service::ToolService;
mod create_session;
mod deserialisers;
mod get_all_topics;
mod get_topics;
mod review_topic;
mod tool_service;
mod topic_candidates;
mod topic_depth;
mod update_session;
mod update_topic_categories;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();

    if has_version_argument(&mut args) {
        print_version();
        return Ok(());
    }

    let io = (tokio::io::stdin(), tokio::io::stdout());

    let conn = SqlConnection::new().await?;
    let arc_conn = Arc::new(conn);
    let topic_storage = SqlTopicStorage::init(arc_conn.clone()).await?;
    let session_storage = SqlSessionStorage::init(arc_conn.clone()).await?;

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
