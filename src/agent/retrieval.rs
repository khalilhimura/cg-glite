use anyhow::Result;
use graphlite_rust_sdk::Session;
use crate::graph::GraphDB;

/// Context retrieval strategies for the agent
pub struct ContextRetriever<'a> {
    graph_db: &'a GraphDB,
}

impl<'a> ContextRetriever<'a> {
    /// Create a new context retriever
    pub fn new(graph_db: &'a GraphDB) -> Self {
        Self { graph_db }
    }

    /// Retrieve context about a specific person
    pub fn get_person_context(&self, session: &Session, person_name: &str) -> Result<String> {
        // Query graph for information about this person
        // This is a placeholder - actual implementation depends on GraphLite query results
        Ok(format!("Context about {}: [To be implemented]", person_name))
    }

    /// Retrieve context about a topic
    pub fn get_topic_context(&self, session: &Session, topic_name: &str) -> Result<String> {
        let related = self.graph_db.find_related_entities(session, topic_name)?;

        if related.is_empty() {
            Ok(format!("No previous context found for topic '{}'", topic_name))
        } else {
            Ok(format!(
                "Topic '{}' is related to: {}",
                topic_name,
                related.join(", ")
            ))
        }
    }

    /// Retrieve recent conversation history
    pub fn get_recent_history(
        &self,
        session: &Session,
        conversation_id: &str,
        limit: usize,
    ) -> Result<String> {
        let messages = self.graph_db.get_conversation_messages(session, conversation_id, limit)?;

        if messages.is_empty() {
            Ok("No recent messages found.".to_string())
        } else {
            let formatted = messages
                .iter()
                .map(|(role, content, timestamp)| {
                    format!("[{}] {}: {}", timestamp, role, content)
                })
                .collect::<Vec<_>>()
                .join("\n");

            Ok(formatted)
        }
    }
}
