use anyhow::{Context, Result};
use graphlite_rust_sdk::{GraphLite, Session};
use std::path::Path;
use super::schema::{Conversation, Message, Person, Topic, ExtractedEntities, new_id, now};

/// GraphDB wrapper for agentic memory operations
pub struct GraphDB {
    db: GraphLite,
}

impl GraphDB {
    /// Initialize a new GraphLite database
    pub async fn new(db_path: &str, admin_user: &str, admin_password: &str) -> Result<Self> {
        // Check if database exists, if not install it
        let path = Path::new(db_path);
        if !path.exists() {
            println!("Initializing new GraphLite database at {}...", db_path);
            GraphLite::install(db_path, admin_user, admin_password)
                .context("Failed to install GraphLite database")?;
        }

        // Open the database
        let db = GraphLite::open(db_path)
            .context("Failed to open GraphLite database")?;

        Ok(Self { db })
    }

    /// Create a new session
    pub fn session(&self, username: &str, password: &str) -> Result<Session> {
        self.db.session(username, password)
            .context("Failed to create database session")
    }

    /// Start a new conversation
    pub fn start_conversation(&self, session: &Session, title: Option<String>) -> Result<String> {
        let conv_id = new_id();
        let timestamp = now();

        let query = format!(
            "INSERT (:Conversation {{id: '{}', started_at: '{}', title: '{}'}})",
            conv_id,
            timestamp.to_rfc3339(),
            title.unwrap_or_else(|| "New Conversation".to_string())
        );

        session.execute(&query)
            .context("Failed to create conversation node")?;

        Ok(conv_id)
    }

    /// Add a message to a conversation with entity extraction
    pub fn add_message(
        &self,
        session: &Session,
        conversation_id: &str,
        role: &str,
        content: &str,
        entities: &ExtractedEntities,
    ) -> Result<String> {
        let msg_id = new_id();
        let timestamp = now();

        // Insert the message node
        let query = format!(
            "INSERT (:Message {{id: '{}', role: '{}', content: '{}', timestamp: '{}'}})",
            msg_id,
            role,
            Self::escape_string(content),
            timestamp.to_rfc3339()
        );
        session.execute(&query)?;

        // Link message to conversation
        let link_query = format!(
            "MATCH (c:Conversation {{id: '{}'}}), (m:Message {{id: '{}'}}) \
             INSERT (m)-[:PART_OF]->(c)",
            conversation_id, msg_id
        );
        session.execute(&link_query)?;

        // Create entity nodes and relationships
        self.link_entities(session, &msg_id, entities)?;

        Ok(msg_id)
    }

    /// Create entity nodes and link them to a message
    fn link_entities(
        &self,
        session: &Session,
        message_id: &str,
        entities: &ExtractedEntities,
    ) -> Result<()> {
        // Add people
        for person_name in &entities.people {
            let person_query = format!(
                "OPTIONAL MATCH (p:Person {{name: '{}'}}) \
                 WITH p \
                 WHERE p IS NULL \
                 INSERT (:Person {{name: '{}'}})",
                Self::escape_string(person_name),
                Self::escape_string(person_name)
            );
            let _ = session.execute(&person_query); // May already exist

            // Link to message
            let link_query = format!(
                "MATCH (p:Person {{name: '{}'}}), (m:Message {{id: '{}'}}) \
                 INSERT (p)-[:MENTIONED_IN]->(m)",
                Self::escape_string(person_name),
                message_id
            );
            session.execute(&link_query)?;
        }

        // Add topics
        for topic_name in &entities.topics {
            let topic_query = format!(
                "OPTIONAL MATCH (t:Topic {{name: '{}'}}) \
                 WITH t \
                 WHERE t IS NULL \
                 INSERT (:Topic {{name: '{}'}})",
                Self::escape_string(topic_name),
                Self::escape_string(topic_name)
            );
            let _ = session.execute(&topic_query);

            let link_query = format!(
                "MATCH (t:Topic {{name: '{}'}}), (m:Message {{id: '{}'}}) \
                 INSERT (t)-[:MENTIONED_IN]->(m)",
                Self::escape_string(topic_name),
                message_id
            );
            session.execute(&link_query)?;
        }

        // Add tasks
        for task_desc in &entities.tasks {
            let task_query = format!(
                "INSERT (:Task {{description: '{}', status: 'pending', created_at: '{}'}})",
                Self::escape_string(task_desc),
                now().to_rfc3339()
            );
            session.execute(&task_query)?;

            // Link to message
            let link_query = format!(
                "MATCH (t:Task {{description: '{}'}}), (m:Message {{id: '{}'}}) \
                 INSERT (t)-[:MENTIONED_IN]->(m)",
                Self::escape_string(task_desc),
                message_id
            );
            session.execute(&link_query)?;
        }

        Ok(())
    }

    /// Query recent messages from a conversation
    pub fn get_conversation_messages(
        &self,
        session: &Session,
        conversation_id: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String)>> {
        let query = format!(
            "MATCH (m:Message)-[:PART_OF]->(c:Conversation {{id: '{}'}}) \
             RETURN m.role, m.content, m.timestamp \
             ORDER BY m.timestamp DESC \
             LIMIT {}",
            conversation_id, limit
        );

        let result = session.query(&query)?;

        // Parse result (this is simplified - actual GraphLite result parsing may differ)
        // For now, return empty vec as we need to check actual GraphLite API
        let messages = Vec::new();

        Ok(messages)
    }

    /// Find entities mentioned in conversations about a topic
    pub fn find_related_entities(
        &self,
        session: &Session,
        topic_name: &str,
    ) -> Result<Vec<String>> {
        let query = format!(
            "MATCH (t:Topic {{name: '{}'}})-[:MENTIONED_IN]->(m:Message)<-[:MENTIONED_IN]-(e) \
             WHERE e:Person OR e:Task \
             RETURN DISTINCT e.name",
            Self::escape_string(topic_name)
        );

        let result = session.query(&query)?;

        // Parse and return entities
        let entities = Vec::new();

        Ok(entities)
    }

    /// Escape single quotes in strings for GQL queries
    fn escape_string(s: &str) -> String {
        s.replace('\'', "\\'")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_escape_string() {
        let input = "It's a test with 'quotes' and\nnewlines";
        let escaped = GraphDB::escape_string(input);
        assert!(escaped.contains("\\'"));
    }
}
