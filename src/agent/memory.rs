use anyhow::{Context, Result};
use crate::graph::{GraphDB, ExtractedEntities};
use crate::llm::{LLMClient, EntityExtractor};
use graphlite_sdk::Session;

/// Agentic memory manager
/// Orchestrates conversation storage, entity extraction, and context building
pub struct AgenticMemory {
    graph_db: GraphDB,
    entity_extractor: EntityExtractor,
    llm_client: LLMClient,
    current_conversation_id: Option<String>,
}

impl AgenticMemory {
    /// Create a new agentic memory instance
    pub async fn new(
        db_path: &str,
        admin_user: &str,
        admin_password: &str,
        llm_client: LLMClient,
    ) -> Result<Self> {
        let graph_db = GraphDB::new(db_path, admin_user, admin_password)
            .await
            .context("Failed to initialize graph database")?;

        let entity_extractor = EntityExtractor::new(llm_client.clone());

        Ok(Self {
            graph_db,
            entity_extractor,
            llm_client,
            current_conversation_id: None,
        })
    }

    /// Get a database session
    pub fn session(&self, username: &str, password: &str) -> Result<Session> {
        self.graph_db.session(username, password)
    }

    /// Start a new conversation
    pub fn start_conversation(&mut self, session: &Session, title: Option<String>) -> Result<String> {
        let conv_id = self.graph_db.start_conversation(session, title)?;
        self.current_conversation_id = Some(conv_id.clone());
        Ok(conv_id)
    }

    /// Get current conversation ID
    pub fn current_conversation(&self) -> Option<&String> {
        self.current_conversation_id.as_ref()
    }

    /// Process and store a user message
    pub async fn process_user_message(
        &self,
        session: &Session,
        message: &str,
    ) -> Result<(String, ExtractedEntities)> {
        let conversation_id = self
            .current_conversation_id
            .as_ref()
            .context("No active conversation")?;

        // Extract entities from the message
        let entities = self
            .entity_extractor
            .extract(message)
            .await
            .context("Failed to extract entities")?;

        // Store the message with entities in the graph
        let msg_id = self
            .graph_db
            .add_message(session, conversation_id, "user", message, &entities)
            .context("Failed to store user message")?;

        Ok((msg_id, entities))
    }

    /// Store an assistant message
    pub fn store_assistant_message(
        &self,
        session: &Session,
        message: &str,
    ) -> Result<String> {
        let conversation_id = self
            .current_conversation_id
            .as_ref()
            .context("No active conversation")?;

        // For assistant messages, we typically don't extract entities
        // but we could if needed
        let entities = ExtractedEntities::default();

        let msg_id = self
            .graph_db
            .add_message(session, conversation_id, "assistant", message, &entities)
            .context("Failed to store assistant message")?;

        Ok(msg_id)
    }

    /// Generate a response using the LLM with context from the graph
    pub async fn generate_response(
        &self,
        session: &Session,
        user_message: &str,
        entities: &ExtractedEntities,
    ) -> Result<String> {
        // Build context from the graph based on extracted entities
        let context = self.build_context(session, entities)?;

        // Construct the system prompt with context
        let system_prompt = format!(
            r#"You are a helpful AI assistant with persistent memory powered by a context graph.

CONTEXT FROM YOUR MEMORY:
{}

Use this context to provide informed, personalized responses. Reference relevant information
from your memory when appropriate. If you remember something about people, topics, or past
conversations mentioned, incorporate that knowledge naturally.

Be conversational and helpful while demonstrating that you remember and understand the
connections between different pieces of information."#,
            context
        );

        // Generate response
        let response = self
            .llm_client
            .complete(&system_prompt, user_message)
            .await
            .context("Failed to generate response")?;

        Ok(response)
    }

    /// Build context from graph based on extracted entities
    fn build_context(&self, session: &Session, entities: &ExtractedEntities) -> Result<String> {
        let mut context_parts = Vec::new();

        // Add information about mentioned people
        if !entities.people.is_empty() {
            context_parts.push(format!(
                "People mentioned: {}",
                entities.people.join(", ")
            ));
        }

        // Add information about mentioned topics
        if !entities.topics.is_empty() {
            context_parts.push(format!(
                "Topics discussed: {}",
                entities.topics.join(", ")
            ));

            // For each topic, try to find related entities
            for topic in &entities.topics {
                if let Ok(related) = self.graph_db.find_related_entities(session, topic) {
                    if !related.is_empty() {
                        context_parts.push(format!(
                            "Related to '{}': {}",
                            topic,
                            related.join(", ")
                        ));
                    }
                }
            }
        }

        // Add information about tasks
        if !entities.tasks.is_empty() {
            context_parts.push(format!(
                "Tasks mentioned: {}",
                entities.tasks.join(", ")
            ));
        }

        if context_parts.is_empty() {
            Ok("No specific context from previous conversations.".to_string())
        } else {
            Ok(context_parts.join("\n"))
        }
    }

    /// Get access to the graph database for custom queries
    pub fn graph(&self) -> &GraphDB {
        &self.graph_db
    }
}
