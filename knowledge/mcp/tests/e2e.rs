use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::serde_json::{Value, json};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;

use rmcp::{RoleClient, ServiceExt};
use serde::Deserialize;
use tempfile::TempDir;
use tokio::process::Command;

#[derive(Deserialize)]
struct TopicDepthResult {
    name: String,
    question_depth: String, // or whatever QuestionDepth deserializes to
}

#[derive(Deserialize)]
pub struct TopicCandidate {
    pub name: String,
    pub days_since_last_review: u64,
}

struct TestClientWrapper {
    client: RunningService<RoleClient, ()>,
    _db_folder: TempDir,
}

async fn create_client() -> TestClientWrapper {
    let binary = std::env::var("BINARY_PATH").unwrap_or("../target/release/mcp".to_string());

    let mut command = Command::new(binary);
    let tmp = TempDir::new().unwrap();
    command.env("AGENT_MENTOR_DB_FOLDER", tmp.path());

    let process = TokioChildProcess::new(command).unwrap();
    let client = ().serve(process).await.unwrap();
    TestClientWrapper {
        client,
        _db_folder: tmp,
    }
}

#[tokio::test]
async fn list_tools() {
    let client = create_client().await;

    let results = client.client.list_all_tools().await.unwrap();

    let mut names = results.iter().map(|t| &t.name).collect::<Vec<_>>();

    names.sort();

    let mut expected = vec![
        "get_topics",
        "topic_depth",
        "review_topic",
        "get_topic_candidates",
        "create_session",
    ];
    expected.sort();

    assert_eq!(names, expected);
}

async fn call_tool(client: &TestClientWrapper, tool_name: String, v: &Value) -> CallToolResult {
    client
        .client
        .call_tool(
            CallToolRequestParams::new(tool_name).with_arguments(v.as_object().unwrap().clone()),
        )
        .await
        .unwrap()
}
#[tokio::test]
async fn update_topic_and_list_them() {
    let client = create_client().await;

    let topic = json!({
        "topic": "some topic",
        "quality": 3
    });

    call_tool(&client, "review_topic".to_string(), &topic).await;

    let topic_update = json!({
        "topic": "some topic",
        "quality": 4
    });

    call_tool(&client, "review_topic".to_string(), &topic_update).await;

    let search = json!({
        "search": "some",
    });

    let result = call_tool(&client, "get_topics".to_string(), &search).await;

    let topics = result.into_typed::<Vec<String>>().unwrap();

    assert_eq!(topics, vec!["some topic"])
}

#[tokio::test]
async fn topic_depth() {
    let client = create_client().await;

    let topic = json!({
        "topic": "some topic",
        "quality": 1
    });

    call_tool(&client, "review_topic".to_string(), &topic).await;

    let topic_depth = json!({
        "topic": "some topic"
    });

    let result = call_tool(&client, "topic_depth".to_string(), &topic_depth).await;

    let topic_depth_result = result.into_typed::<TopicDepthResult>().unwrap();

    assert_eq!(topic_depth_result.question_depth, "full".to_string());
    assert_eq!(topic_depth_result.name, "some topic".to_string());
}

#[tokio::test]
async fn get_topic_candidates() {
    let client = create_client().await;

    let topic = json!({
        "topic": "some topic",
        "quality": 0
    });

    call_tool(&client, "review_topic".to_string(), &topic).await;

    let topic_depth = json!({
        "limit": 1
    });

    let result = call_tool(&client, "get_topic_candidates".to_string(), &topic_depth).await;

    let candidates = result.into_typed::<Vec<TopicCandidate>>().unwrap();

    assert_eq!(candidates.len(), 0);
}
