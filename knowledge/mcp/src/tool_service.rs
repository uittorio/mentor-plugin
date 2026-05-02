use chrono::Utc;
use learning::session::{Session, SessionId};
use learning::session_storage::SessionStorage;
use learning::topic::{QuestionDepth, Topic};
use learning::topic_storage::TopicStorage;
use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    serde_json::{self},
    tool, tool_handler, tool_router,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::create_session::{CreateSessionParams, CreateSessionResult};
use crate::get_all_sessions::{GetAllSessionsParams, SessionResult};
use crate::get_all_topics::TopicResult;
use crate::update_session::{UpdateSessionParams, UpdateSessionResult};
use crate::update_topic_categories::{UpdateTopicCategoriesParams, UpdateTopicCategoriesResult};
use crate::{
    get_topics::GetTopicsParams,
    review_topic::{ReviewTopicParams, ReviewTopicResult},
    topic_candidates::{TopicCandidate, TopicCandidatesParams},
    topic_depth::{TopicDepthParams, TopicDepthResult},
};

pub struct ToolService {
    tool_router: ToolRouter<Self>,
    topic_storage: Box<dyn TopicStorage + Send + Sync>,
    session_storage: Box<dyn SessionStorage + Send + Sync>,
}

fn normalise_topic(name: &str) -> String {
    name.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

#[tool_router]
impl ToolService {
    pub fn new(
        topic_storage: Box<dyn TopicStorage + Send + Sync>,
        session_storage: Box<dyn SessionStorage + Send + Sync>,
    ) -> Self {
        Self {
            tool_router: Self::tool_router(),
            topic_storage,
            session_storage,
        }
    }

    #[tool(
        description = "Search for existing topic names similar to a search string using trigram similarity. Returns topic names ranked by similarity."
    )]
    async fn get_topics(&self, params: Parameters<GetTopicsParams>) -> Result<String, String> {
        let topics = self
            .topic_storage
            .get_all()
            .await
            .map_err(|e| e.to_string())?;
        let topic_names = topics
            .into_iter()
            .filter(|t| t.is_similar(&params.0.search))
            .map(|x| x.name)
            .collect::<Vec<String>>();

        serde_json::to_string(&topic_names).map_err(|e| e.to_string())
    }

    #[tool(description = "Get all topics")]
    async fn get_all_topics(&self) -> Result<String, String> {
        let topics = self
            .topic_storage
            .get_all()
            .await
            .map_err(|e| e.to_string())?;
        let topic_names = topics
            .into_iter()
            .map(|x| TopicResult {
                name: x.name,
                categories: x.categories.0.iter().map(|c| c.name.clone()).collect(),
            })
            .collect::<Vec<TopicResult>>();

        serde_json::to_string(&topic_names).map_err(|e| e.to_string())
    }

    #[tool(
        description = "Returns the recommended question depth (full/light/skip) for a single topic. Call once per topic after resolving the canonical name with get_topics."
    )]
    async fn topic_depth(&self, params: Parameters<TopicDepthParams>) -> Result<String, String> {
        let topic_name = normalise_topic(&params.0.topic);
        let topic = self
            .topic_storage
            .get(&topic_name)
            .await
            .map_err(|e| e.to_string())?;

        let topic_depth = match topic {
            Some(t) => TopicDepthResult {
                name: t.name.to_string(),
                question_depth: t.question_depth(),
            },
            None => TopicDepthResult {
                name: topic_name.to_string(),
                question_depth: QuestionDepth::Full,
            },
        };

        serde_json::to_string(&topic_depth).map_err(|e| e.to_string())
    }

    #[tool(description = "Update the topic's knowledge record after a learning exchange.")]
    async fn review_topic(&self, params: Parameters<ReviewTopicParams>) -> Result<String, String> {
        let topic_name = normalise_topic(&params.0.topic);

        let quality = params.0.quality as u32;
        let now = Utc::now();
        let topic = self
            .topic_storage
            .get(&topic_name)
            .await
            .map_err(|e| e.to_string())?
            .unwrap_or(Topic::new(&topic_name, now));

        let updated_topic = topic.update_quality(quality as u32, now);

        self.topic_storage
            .upsert(&updated_topic)
            .await
            .map_err(|e| e.to_string())?;

        let result = ReviewTopicResult {
            topic: updated_topic.name,
            next_review_in_days: updated_topic.interval,
        };

        serde_json::to_string(&result).map_err(|e| e.to_string())
    }

    #[tool(description = "Update the topic's categories.")]
    async fn update_topic_categories(
        &self,
        params: Parameters<UpdateTopicCategoriesParams>,
    ) -> Result<String, String> {
        let topic_name = normalise_topic(&params.0.topic);

        let topic = self
            .topic_storage
            .get(&topic_name)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Topic not found")?;

        let updated_topic = topic.update_categories(params.0.categories);

        self.topic_storage
            .upsert(&updated_topic)
            .await
            .map_err(|e| e.to_string())?;

        let result = UpdateTopicCategoriesResult {
            topic: updated_topic.name,
            categories: updated_topic
                .categories
                .0
                .iter()
                .map(|c| c.name.clone())
                .collect(),
        };

        serde_json::to_string(&result).map_err(|e| e.to_string())
    }

    #[tool(description = "Get topics that are due for review based on SM-2 intervals.")]
    async fn get_topic_candidates(
        &self,
        params: Parameters<TopicCandidatesParams>,
    ) -> Result<String, String> {
        let now = Utc::now();

        let results = self
            .topic_storage
            .get_overdue(now)
            .await
            .map_err(|e| e.to_string())?
            .iter()
            .take(params.0.limit)
            .map(|t| TopicCandidate {
                name: t.name.clone(),
                days_since_last_review: t.days_since_last_review(now),
            })
            .collect::<Vec<TopicCandidate>>();

        serde_json::to_string(&results).map_err(|e| e.to_string())
    }

    #[tool(
        description = "Create a new session in order to store the summary and the tree of socratic questions and answers."
    )]
    async fn create_session(
        &self,
        params: Parameters<CreateSessionParams>,
    ) -> Result<String, String> {
        let now = Utc::now();

        let name = &params.0.name;

        let session = Session::new(&name, now);

        self.session_storage
            .create(&session)
            .await
            .map_err(|e| e.to_string())?;

        let result = CreateSessionResult {
            session_id: session.id.0.to_string(),
            session_name: session.name,
        };

        serde_json::to_string(&result).map_err(|e| e.to_string())
    }

    #[tool(description = "Update a session content")]
    async fn update_session(
        &self,
        params: Parameters<UpdateSessionParams>,
    ) -> Result<String, String> {
        let uuid = Uuid::from_str(params.0.session_id.as_str()).map_err(|e| e.to_string())?;
        let session_id = SessionId(uuid);

        let session = self
            .session_storage
            .get(&session_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Topic not found")?;

        let session = session.update_content(&params.0.content, Utc::now());

        self.session_storage
            .update(&session)
            .await
            .map_err(|e| e.to_string())?;

        let result = UpdateSessionResult {
            session_name: session.name,
        };

        serde_json::to_string(&result).map_err(|e| e.to_string())
    }

    #[tool(description = "Get all sessions")]
    async fn get_all_sessions(
        &self,
        params: Parameters<GetAllSessionsParams>,
    ) -> Result<String, String> {
        let sessions = self
            .session_storage
            .get_all()
            .await
            .map_err(|e| e.to_string())?
            .iter()
            .take(params.0.limit)
            .map(|s| SessionResult {
                name: s.name.clone(),
                id: s.id.0.to_string(),
            })
            .collect::<Vec<_>>();

        serde_json::to_string(&sessions).map_err(|e| e.to_string())
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
