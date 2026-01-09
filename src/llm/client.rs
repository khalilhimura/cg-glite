use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// LLM provider type
#[derive(Debug, Clone)]
pub enum LLMProvider {
    OpenAI { api_key: String, model: String },
    Anthropic { api_key: String, model: String },
    OpenRouter {
        api_key: String,
        model: String,
        app_name: Option<String>,
        site_url: Option<String>,
    },
}

/// LLM client for making API calls
#[derive(Clone)]
pub struct LLMClient {
    provider: LLMProvider,
    http_client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

/// Common structure for LLM API requests
struct LLMRequest {
    url: String,
    headers: Vec<(String, String)>,
    body: serde_json::Value,
}

impl LLMRequest {
    fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
            body: json!({}),
        }
    }

    fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    fn body(mut self, body: serde_json::Value) -> Self {
        self.body = body;
        self
    }
}

/// Trait for provider-specific response parsing
trait ResponseParser {
    fn parse_completion(&self, response_text: &str) -> Result<String>;
}

/// Anthropic response parser
struct AnthropicParser;

impl ResponseParser for AnthropicParser {
    fn parse_completion(&self, response_text: &str) -> Result<String> {
        let response: AnthropicResponse = serde_json::from_str(response_text)
            .context("Failed to parse Anthropic response")?;
        Ok(response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }
}

/// OpenAI response parser (also used for OpenRouter)
struct OpenAIParser;

impl ResponseParser for OpenAIParser {
    fn parse_completion(&self, response_text: &str) -> Result<String> {
        let response: OpenAIResponse = serde_json::from_str(response_text)
            .context("Failed to parse OpenAI response")?;
        Ok(response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}

impl LLMClient {
    /// Create a new LLM client
    pub fn new(provider: LLMProvider) -> Self {
        Self {
            provider,
            http_client: Client::new(),
        }
    }

    /// Send a completion request to the LLM
    pub async fn complete(&self, system: &str, user_message: &str) -> Result<String> {
        match &self.provider {
            LLMProvider::Anthropic { api_key, model } => {
                self.anthropic_complete(api_key, model, system, user_message)
                    .await
            }
            LLMProvider::OpenAI { api_key, model } => {
                self.openai_complete(api_key, model, system, user_message)
                    .await
            }
            LLMProvider::OpenRouter { api_key, model, app_name, site_url } => {
                self.openrouter_complete(api_key, model, app_name.as_deref(), site_url.as_deref(), system, user_message)
                    .await
            }
        }
    }

    /// Execute an LLM request and parse the response
    async fn execute_request(
        &self,
        request: LLMRequest,
        parser: &dyn ResponseParser,
        provider_name: &str,
    ) -> Result<String> {
        let mut http_request = self
            .http_client
            .post(&request.url)
            .header("content-type", "application/json");

        // Add custom headers
        for (key, value) in request.headers {
            http_request = http_request.header(key, value);
        }

        // Send request
        let response = http_request
            .json(&request.body)
            .send()
            .await
            .context(format!("Failed to send request to {} API", provider_name))?;

        // Check status
        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("{} API error ({}): {}", provider_name, status, response_text);
        }

        // Parse response
        parser.parse_completion(&response_text)
    }

    /// Anthropic API completion
    async fn anthropic_complete(
        &self,
        api_key: &str,
        model: &str,
        system: &str,
        user_message: &str,
    ) -> Result<String> {
        let request = LLMRequest::new("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .body(json!({
                "model": model,
                "max_tokens": 4096,
                "system": system,
                "messages": [
                    {
                        "role": "user",
                        "content": user_message
                    }
                ]
            }));

        self.execute_request(request, &AnthropicParser, "Anthropic")
            .await
    }

    /// OpenAI API completion
    async fn openai_complete(
        &self,
        api_key: &str,
        model: &str,
        system: &str,
        user_message: &str,
    ) -> Result<String> {
        let request = LLMRequest::new("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .body(json!({
                "model": model,
                "temperature": 0.7,
                "messages": [
                    {
                        "role": "system",
                        "content": system
                    },
                    {
                        "role": "user",
                        "content": user_message
                    }
                ]
            }));

        self.execute_request(request, &OpenAIParser, "OpenAI")
            .await
    }

    /// OpenRouter API completion (OpenAI-compatible format)
    async fn openrouter_complete(
        &self,
        api_key: &str,
        model: &str,
        app_name: Option<&str>,
        site_url: Option<&str>,
        system: &str,
        user_message: &str,
    ) -> Result<String> {
        let mut request = LLMRequest::new("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .body(json!({
                "model": model,
                "temperature": 0.7,
                "messages": [
                    {
                        "role": "system",
                        "content": system
                    },
                    {
                        "role": "user",
                        "content": user_message
                    }
                ]
            }));

        // Add optional headers for app tracking
        if let Some(name) = app_name {
            request = request.header("X-Title", name);
        }
        if let Some(url) = site_url {
            request = request.header("HTTP-Referer", url);
        }

        self.execute_request(request, &OpenAIParser, "OpenRouter")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_request_builder() {
        let request = LLMRequest::new("https://api.example.com")
            .header("Authorization", "Bearer token")
            .header("Custom-Header", "value")
            .body(json!({"key": "value"}));

        assert_eq!(request.url, "https://api.example.com");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.headers[0].0, "Authorization");
        assert_eq!(request.headers[0].1, "Bearer token");
        assert_eq!(request.headers[1].0, "Custom-Header");
        assert_eq!(request.headers[1].1, "value");
        assert_eq!(request.body["key"], "value");
    }

    #[test]
    fn test_anthropic_parser() {
        let parser = AnthropicParser;
        let response = r#"{"content": [{"text": "Hello, world!"}]}"#;
        let result = parser.parse_completion(response).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_anthropic_parser_empty_content() {
        let parser = AnthropicParser;
        let response = r#"{"content": []}"#;
        let result = parser.parse_completion(response).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_anthropic_parser_invalid_json() {
        let parser = AnthropicParser;
        let response = "not valid json";
        assert!(parser.parse_completion(response).is_err());
    }

    #[test]
    fn test_openai_parser() {
        let parser = OpenAIParser;
        let response = r#"{"choices": [{"message": {"role": "assistant", "content": "Hello, world!"}}]}"#;
        let result = parser.parse_completion(response).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_openai_parser_empty_choices() {
        let parser = OpenAIParser;
        let response = r#"{"choices": []}"#;
        let result = parser.parse_completion(response).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_openai_parser_invalid_json() {
        let parser = OpenAIParser;
        let response = "not valid json";
        assert!(parser.parse_completion(response).is_err());
    }

    #[test]
    fn test_llm_request_chaining() {
        let request = LLMRequest::new("https://test.com")
            .header("Header1", "Value1")
            .body(json!({"test": true}))
            .header("Header2", "Value2");

        assert_eq!(request.headers.len(), 2);
        assert!(request.body["test"].as_bool().unwrap());
    }
}
