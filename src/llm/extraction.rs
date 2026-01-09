use anyhow::{Context, Result};
use serde_json::json;
use super::client::LLMClient;
use crate::graph::schema::ExtractedEntities;

/// Entity extractor using LLM
pub struct EntityExtractor {
    llm_client: LLMClient,
}

impl EntityExtractor {
    /// Create a new entity extractor
    pub fn new(llm_client: LLMClient) -> Self {
        Self { llm_client }
    }

    /// Extract entities from a user message
    pub async fn extract(&self, message: &str) -> Result<ExtractedEntities> {
        let system_prompt = r#"You are an expert entity extractor for an AI agent's memory system.
Extract the following types of entities from the user's message:

1. PEOPLE: Names of individuals mentioned
2. TOPICS: Subjects, concepts, technologies, projects, or areas of interest
3. TASKS: Action items, todos, or work that needs to be done
4. DOCUMENTS: Files, links, resources, or references mentioned

Return your response as a JSON object with this exact structure:
{
  "people": ["name1", "name2"],
  "topics": ["topic1", "topic2"],
  "tasks": ["task1", "task2"],
  "documents": ["doc1", "doc2"]
}

Guidelines:
- Only extract entities that are explicitly mentioned or clearly implied
- For topics, include both specific technologies and general concepts
- For tasks, extract actionable items in imperative form
- If a category has no entities, use an empty array []
- Be precise and avoid over-extraction

Return ONLY the JSON object, no additional text."#;

        let response = self
            .llm_client
            .complete(system_prompt, message)
            .await
            .context("Failed to extract entities from message")?;

        // Parse JSON response
        let entities = self.parse_extraction_response(&response)?;

        Ok(entities)
    }

    /// Extract a string array from a JSON value, returning empty vec if not present
    fn extract_string_array(value: &serde_json::Value, field_name: &str) -> Vec<String> {
        value[field_name]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse the LLM's extraction response
    fn parse_extraction_response(&self, response: &str) -> Result<ExtractedEntities> {
        // Try to extract JSON from the response (handle cases where LLM adds extra text)
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .context("Failed to parse entity extraction JSON")?;

        let entities = ExtractedEntities {
            people: Self::extract_string_array(&parsed, "people"),
            topics: Self::extract_string_array(&parsed, "topics"),
            tasks: Self::extract_string_array(&parsed, "tasks"),
            documents: Self::extract_string_array(&parsed, "documents"),
        };

        Ok(entities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_string_array() {
        let json = json!({
            "field": ["a", "b", "c"]
        });
        let result = EntityExtractor::extract_string_array(&json, "field");
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_extract_string_array_missing_field() {
        let json = json!({});
        let result = EntityExtractor::extract_string_array(&json, "field");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_extract_string_array_null_value() {
        let json = json!({
            "field": null
        });
        let result = EntityExtractor::extract_string_array(&json, "field");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_extract_string_array_empty_array() {
        let json = json!({
            "field": []
        });
        let result = EntityExtractor::extract_string_array(&json, "field");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_extract_string_array_mixed_types() {
        let json = json!({
            "field": ["string", 123, true, null, "another"]
        });
        let result = EntityExtractor::extract_string_array(&json, "field");
        // Should only extract string values
        assert_eq!(result, vec!["string", "another"]);
    }

    #[test]
    fn test_parse_extraction_response() {
        let extractor = EntityExtractor {
            llm_client: LLMClient::new(crate::llm::LLMProvider::Anthropic {
                api_key: "test".to_string(),
                model: "test".to_string(),
            }),
        };

        let response = r#"{
  "people": ["Alice", "Bob"],
  "topics": ["GraphQL", "API Design"],
  "tasks": ["Review PR", "Update docs"],
  "documents": []
}"#;

        let entities = extractor.parse_extraction_response(response).unwrap();

        assert_eq!(entities.people.len(), 2);
        assert_eq!(entities.topics.len(), 2);
        assert_eq!(entities.tasks.len(), 2);
        assert_eq!(entities.documents.len(), 0);
    }
}
