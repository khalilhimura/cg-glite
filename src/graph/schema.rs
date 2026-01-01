use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Graph schema initialization queries for GraphLite
/// This defines the Context Graph structure for agentic memory

/// Node type: Conversation
/// Represents a conversation session with the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub title: Option<String>,
}

/// Node type: Message
/// Represents a single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Node type: Person
/// Represents a person mentioned in conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub description: Option<String>,
}

/// Node type: Topic
/// Represents a topic, concept, or technology discussed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub category: Option<String>,
}

/// Node type: Task
/// Represents an action item or work task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub description: String,
    pub status: String, // "pending", "in_progress", "completed"
    pub created_at: DateTime<Utc>,
}

/// Node type: Document
/// Represents a file, link, or resource referenced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub url: Option<String>,
    pub doc_type: String, // "file", "link", "reference"
}

/// Extracted entities from a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntities {
    pub people: Vec<String>,
    pub topics: Vec<String>,
    pub tasks: Vec<String>,
    pub documents: Vec<String>,
}

impl Default for ExtractedEntities {
    fn default() -> Self {
        Self {
            people: Vec::new(),
            topics: Vec::new(),
            tasks: Vec::new(),
            documents: Vec::new(),
        }
    }
}

/// Schema initialization for GraphLite database
/// Creates node labels and constraints
pub fn get_schema_init_queries() -> Vec<String> {
    vec![
        // Note: GraphLite uses GQL standard, which may have different syntax
        // These are conceptual queries that may need adjustment based on actual GraphLite GQL support

        // Create unique constraints (if supported)
        // In GQL, we typically just use INSERT to create nodes
        // Constraints might be handled differently

        String::from("// Schema initialization"),
        String::from("// Nodes will be created dynamically during operation"),
        String::from("// GraphLite uses ISO GQL standard for queries"),
    ]
}

/// Helper to generate a new UUID as string
pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

/// Helper to get current UTC timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}
