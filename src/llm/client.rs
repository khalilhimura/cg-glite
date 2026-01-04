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

    /// Anthropic API completion
    async fn anthropic_complete(
        &self,
        api_key: &str,
        model: &str,
        system: &str,
        user_message: &str,
    ) -> Result<String> {
        let url = "https://api.anthropic.com/v1/messages";

        let request_body = json!({
            "model": model,
            "max_tokens": 4096,
            "system": system,
            "messages": [
                {
                    "role": "user",
                    "content": user_message
                }
            ]
        });

        let response = self
            .http_client
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("Anthropic API error ({}): {}", status, response_text);
        }

        let anthropic_response: AnthropicResponse = serde_json::from_str(&response_text)
            .context("Failed to parse Anthropic response")?;

        Ok(anthropic_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }

    /// OpenAI API completion
    async fn openai_complete(
        &self,
        api_key: &str,
        model: &str,
        system: &str,
        user_message: &str,
    ) -> Result<String> {
        let url = "https://api.openai.com/v1/chat/completions";

        let request_body = OpenAIRequest {
            model: model.to_string(),
            temperature: 0.7,
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
        };

        let response = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("OpenAI API error ({}): {}", status, response_text);
        }

        let openai_response: OpenAIResponse = serde_json::from_str(&response_text)
            .context("Failed to parse OpenAI response")?;

        Ok(openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
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
        let url = "https://openrouter.ai/api/v1/chat/completions";

        let request_body = OpenAIRequest {
            model: model.to_string(),
            temperature: 0.7,
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
        };

        let mut request = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json");

        // Add optional headers for app tracking
        if let Some(name) = app_name {
            request = request.header("X-Title", name);
        }
        if let Some(url) = site_url {
            request = request.header("HTTP-Referer", url);
        }

        let response = request
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to OpenRouter API")?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("OpenRouter API error ({}): {}", status, response_text);
        }

        let openrouter_response: OpenAIResponse = serde_json::from_str(&response_text)
            .context("Failed to parse OpenRouter response")?;

        Ok(openrouter_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}
