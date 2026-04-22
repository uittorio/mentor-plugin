use learning::file_storage::session_file_name;
use learning::session::Session;
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
use std::time::SystemTimeError;

use crate::create_session::{CreateSessionParams, CreateSessionResult};
use crate::get_all_topics::TopicResult;
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

    fn now_epoch(&self) -> Result<u64, SystemTimeError> {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|t| t.as_secs())
    }

    #[tool(
        description = "Search for existing topic names similar to a search string using trigram similarity. Returns topic names ranked by similarity."
    )]
    async fn get_topics(&self, params: Parameters<GetTopicsParams>) -> Result<String, String> {
        let topics = self.topic_storage.get_all().map_err(|e| e.to_string())?;
        let topic_names = topics
            .into_iter()
            .filter(|t| t.is_similar(&params.0.search))
            .map(|x| x.name)
            .collect::<Vec<String>>();

        serde_json::to_string(&topic_names).map_err(|e| e.to_string())
    }

    #[tool(description = "Get all topics")]
    async fn get_all_topics(&self) -> Result<String, String> {
        let topics = self.topic_storage.get_all().map_err(|e| e.to_string())?;
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
        let epoch_now = self.now_epoch().map_err(|e| e.to_string())?;
        let topic = self
            .topic_storage
            .get(&topic_name)
            .map_err(|e| e.to_string())?
            .unwrap_or(Topic::new(&topic_name, epoch_now));

        let updated_topic = topic.update_quality(quality as u32, epoch_now);

        self.topic_storage
            .upsert(&updated_topic)
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
            .map_err(|e| e.to_string())?
            .ok_or("Topic not found")?;

        let updated_topic = topic.update_categories(params.0.categories);

        self.topic_storage
            .upsert(&updated_topic)
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
        let epoch_now = self.now_epoch().map_err(|e| e.to_string())?;

        let results = self
            .topic_storage
            .get_overdue(epoch_now)
            .map_err(|e| e.to_string())?
            .iter()
            .take(params.0.limit)
            .map(|t| TopicCandidate {
                name: t.name.clone(),
                days_since_last_review: t.days_since_last_review(epoch_now),
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
        let epoch_now = self.now_epoch().map_err(|e| e.to_string())?;

        let name = &params.0.name;

        let session = Session::new(&name, &session_file_name(name), epoch_now);

        self.session_storage
            .create(&session)
            .map_err(|e| e.to_string())?;

        let session_file_path = session
            .file_path()
            .expect("path should be present as it was created now");
        let result = CreateSessionResult {
            session_id: session.id.0.to_string(),
            session_file_path,
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
