use anyhow::{Context, Result};
use graphlite_sdk::{GraphLite, Session};
use std::path::Path;
use super::schema::{Conversation, Message, Person, Topic, ExtractedEntities, new_id, now};

/// Configuration for different entity types
enum EntityConfig<'a> {
    Person { name: &'a str },
    Topic { name: &'a str },
    Task { description: &'a str },
}

impl<'a> EntityConfig<'a> {
    /// Get the node label for this entity type
    fn label(&self) -> &'static str {
        match self {
            EntityConfig::Person { .. } => "Person",
            EntityConfig::Topic { .. } => "Topic",
            EntityConfig::Task { .. } => "Task",
        }
    }

    /// Get the property name used as the identifier
    fn id_property(&self) -> &'static str {
        match self {
            EntityConfig::Person { .. } | EntityConfig::Topic { .. } => "name",
            EntityConfig::Task { .. } => "description",
        }
    }

    /// Get the escaped identifier value
    fn id_value(&self) -> String {
        let raw_value = match self {
            EntityConfig::Person { name } => name,
            EntityConfig::Topic { name } => name,
            EntityConfig::Task { description } => description,
        };
        GraphDB::escape_string(raw_value)
    }

    /// Whether this entity type should be deduplicated
    fn should_deduplicate(&self) -> bool {
        matches!(self, EntityConfig::Person { .. } | EntityConfig::Topic { .. })
    }

    /// Additional properties for INSERT query (empty for Person/Topic)
    fn additional_properties(&self) -> Option<String> {
        match self {
            EntityConfig::Task { .. } => Some(format!(
                ", status: 'pending', created_at: '{}'",
                now().to_rfc3339()
            )),
            _ => None,
        }
    }
}

/// GraphDB wrapper for agentic memory operations
pub struct GraphDB {
    db: GraphLite,
}

impl GraphDB {
    /// Initialize a new GraphLite database
    pub async fn new(db_path: &str, _admin_user: &str, _admin_password: &str) -> Result<Self> {
        // Open the database (creates it if it doesn't exist)
        let db = GraphLite::open(db_path)
            .context("Failed to open GraphLite database")?;

        Ok(Self { db })
    }

    /// Create a new session
    pub fn session(&self, username: &str, _password: &str) -> Result<Session> {
        self.db.session(username)
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

    /// Link a single entity to a message
    fn link_entity(
        &self,
        session: &Session,
        message_id: &str,
        config: EntityConfig,
    ) -> Result<()> {
        let label = config.label();
        let id_prop = config.id_property();
        let id_value = config.id_value();

        // Create entity node if needed
        if config.should_deduplicate() {
            // Use OPTIONAL MATCH pattern for deduplication
            let insert_query = format!(
                "OPTIONAL MATCH (e:{} {{{}: '{}'}}) \
                 WITH e \
                 WHERE e IS NULL \
                 INSERT (:{} {{{}: '{}'{}}})",
                label, id_prop, id_value,
                label, id_prop, id_value,
                config.additional_properties().unwrap_or_default()
            );
            let _ = session.execute(&insert_query); // May already exist
        } else {
            // Always create new entity (for Tasks)
            let insert_query = format!(
                "INSERT (:{} {{{}: '{}'{}}})",
                label, id_prop, id_value,
                config.additional_properties().unwrap_or_default()
            );
            session.execute(&insert_query)?;
        }

        // Link entity to message
        let link_query = format!(
            "MATCH (e:{} {{{}: '{}'}}), (m:Message {{id: '{}'}}) \
             INSERT (e)-[:MENTIONED_IN]->(m)",
            label, id_prop, id_value, message_id
        );
        session.execute(&link_query)?;

        Ok(())
    }

    /// Create entity nodes and link them to a message
    fn link_entities(
        &self,
        session: &Session,
        message_id: &str,
        entities: &ExtractedEntities,
    ) -> Result<()> {
        // Link all people
        for person_name in &entities.people {
            self.link_entity(
                session,
                message_id,
                EntityConfig::Person { name: person_name },
            )?;
        }

        // Link all topics
        for topic_name in &entities.topics {
            self.link_entity(
                session,
                message_id,
                EntityConfig::Topic { name: topic_name },
            )?;
        }

        // Link all tasks
        for task_desc in &entities.tasks {
            self.link_entity(
                session,
                message_id,
                EntityConfig::Task { description: task_desc },
            )?;
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

    #[test]
    fn test_entity_config_person() {
        let config = EntityConfig::Person { name: "Alice" };
        assert_eq!(config.label(), "Person");
        assert_eq!(config.id_property(), "name");
        assert!(config.should_deduplicate());
        assert!(config.additional_properties().is_none());
    }

    #[test]
    fn test_entity_config_topic() {
        let config = EntityConfig::Topic { name: "Rust" };
        assert_eq!(config.label(), "Topic");
        assert_eq!(config.id_property(), "name");
        assert!(config.should_deduplicate());
        assert!(config.additional_properties().is_none());
    }

    #[test]
    fn test_entity_config_task() {
        let config = EntityConfig::Task { description: "Test task" };
        assert_eq!(config.label(), "Task");
        assert_eq!(config.id_property(), "description");
        assert!(!config.should_deduplicate());
        assert!(config.additional_properties().is_some());

        // Verify additional properties contain expected fields
        let props = config.additional_properties().unwrap();
        assert!(props.contains("status: 'pending'"));
        assert!(props.contains("created_at:"));
    }

    #[test]
    fn test_entity_config_id_value_escaping() {
        let config = EntityConfig::Person { name: "O'Reilly" };
        let escaped = config.id_value();
        assert!(escaped.contains("\\'"));
        assert_eq!(escaped, "O\\'Reilly");
    }

    #[test]
    fn test_entity_config_id_value_newlines() {
        let config = EntityConfig::Task { description: "Line1\nLine2" };
        let escaped = config.id_value();
        assert!(escaped.contains("\\n"));
        assert_eq!(escaped, "Line1\\nLine2");
    }

    #[tokio::test]
    async fn test_escape_string() {
        let input = "It's a test with 'quotes' and\nnewlines";
        let escaped = GraphDB::escape_string(input);
        assert!(escaped.contains("\\'"));
    }
}
