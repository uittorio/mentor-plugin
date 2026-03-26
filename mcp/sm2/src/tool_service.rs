use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    schemars, serde_json, tool, tool_handler, tool_router,
};

#[derive(Clone)]
pub struct ToolService {
    tool_router: ToolRouter<Self>,
}

fn deserialize_usize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    use serde::Deserialize;
    match serde_json::Value::deserialize(d)? {
        serde_json::Value::Number(n) => n
            .as_u64()
            .map(|n| n as usize)
            .ok_or_else(|| serde::de::Error::custom("expected non-negative integer")),
        serde_json::Value::String(s) => s.parse().map_err(serde::de::Error::custom),
        other => Err(serde::de::Error::custom(format!(
            "expected integer, got {other}"
        ))),
    }
}

fn deserialize_string_vec<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<Vec<String>>, D::Error> {
    use serde::Deserialize;
    match serde_json::Value::deserialize(d)? {
        serde_json::Value::Array(arr) => arr
            .into_iter()
            .map(|v| match v {
                serde_json::Value::String(s) => Ok(s),
                other => Err(serde::de::Error::custom(format!(
                    "expected string, got {other}"
                ))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        serde_json::Value::String(s) => serde_json::from_str(&s).map_err(serde::de::Error::custom),
        serde_json::Value::Null => Ok(None),
        other => Err(serde::de::Error::custom(format!(
            "expected array, got {other}"
        ))),
    }
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct KnowledgeCheckParams {
    #[serde(default, deserialize_with = "deserialize_string_vec")]
    topics: Option<Vec<String>>,
    topic: Option<String>,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct KnowledgeUpdateParams {
    topic: String,
    #[serde(deserialize_with = "deserialize_usize")]
    quality: usize,
    category: Option<String>,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ReviewCandidatesParams {
    #[serde(deserialize_with = "deserialize_usize")]
    limit: usize,
}

#[derive(serde::Serialize)]
pub struct KnowledgeCheckResult {
    topic: String,
    known: bool,
    confidence: usize,
    question_depth: String, // probably need enum/union
    days_since_review: Option<usize>,
    message: String,
}

#[derive(serde::Serialize)]
pub struct KnowledgeUpdateResult {
    topic: String,
    previous_confidence: usize,
    new_confidence: usize,
    next_review_in_days: usize,
    message: String,
}

#[derive(serde::Serialize)]
pub struct ReviewCandidate {
    topic: String,
    confidence: usize,
    day_since_review: usize,
    message: String,
}

#[tool_router]
impl ToolService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Check the user's confidence level for one or more topics. Returns per-topic scores and recommended question depth (skip/light/full). Call this at the start of a mentor session once you have identified the relevant topics."
    )]
    async fn knowledge_check(
        &self,
        _params: Parameters<KnowledgeCheckParams>,
    ) -> Result<String, String> {
        let result = KnowledgeCheckResult {
            topic: "".to_string(),
            known: true,
            confidence: 2,
            question_depth: "low".to_string(),
            days_since_review: Some(10),
            message: "message".to_string(),
        };
        let results: Vec<KnowledgeCheckResult> = vec![result];
        serde_json::to_string(&results).map_err(|e| e.to_string())
    }

    #[tool(
        description = "Update the user's knowledge record for a topic after a learning exchange. Call this after a meaningful Socratic interaction where you can assess how well the user understood the concept."
    )]
    async fn update_knowledge(
        &self,
        _params: Parameters<KnowledgeUpdateParams>,
    ) -> Result<String, String> {
        let result = KnowledgeUpdateResult {
            topic: "".to_string(),
            message: "message".to_string(),
            previous_confidence: 0,
            new_confidence: 1,
            next_review_in_days: 10,
        };

        serde_json::to_string(&result).map_err(|e| e.to_string())
    }

    #[tool(
        description = "Get topics that are due for review based on SM-2 intervals. Call this at session start to surface concepts the user hasn't revisited in a while."
    )]
    async fn get_review_candidates(
        &self,
        _params: Parameters<ReviewCandidatesParams>,
    ) -> Result<String, String> {
        let result = ReviewCandidate {
            topic: "topic".to_string(),
            confidence: 0,
            day_since_review: 1,
            message: "message".to_string(),
        };

        serde_json::to_string(&result).map_err(|e| e.to_string())
    }
}

#[tool_handler]
impl ServerHandler for ToolService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "SM-2 spaced repetition knowledge tracking for mentor sessions".to_string(),
            )
            .with_server_info(Implementation::new("agent-mentor", "1.0.0"))
    }
}
