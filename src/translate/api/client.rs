//! API客户端实现
//!
//! OpenAI兼容API的HTTP客户端封装。

use super::models::*;
use crate::config::ClientSettings;
use crate::error::{Result, TranslationError};
use reqwest::Client;

/// API客户端
pub struct ApiClient {
    client: Client,
    settings: ClientSettings,
    api_key: String,
}

impl ApiClient {
    /// 创建新的API客户端
    pub fn new(settings: ClientSettings, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(settings.timeout_secs))
            .build()
            .map_err(|e| {
                TranslationError::Translate(crate::error::TranslateError::ApiRequest(e))
            })?;

        Ok(Self {
            client,
            settings,
            api_key,
        })
    }

    /// 发送聊天补全请求
    pub async fn chat_completions(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatCompletionResponse> {
        let request = ChatCompletionRequest {
            model: self.settings.model.clone(),
            messages,
            temperature: Some(self.settings.temperature),
            max_tokens: Some(self.settings.max_tokens),
            stream: Some(self.settings.stream),
        };

        let response = self
            .client
            .post(&self.settings.chat_completions_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                TranslationError::Translate(crate::error::TranslateError::ApiRequest(e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(TranslationError::ApiError(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let completion = response.json().await;
        // dbg!(&completion);
        let completion = completion.map_err(|e| {
            TranslationError::Translate(crate::error::TranslateError::InvalidResponse(
                e.to_string(),
            ))
        })?;

        Ok(completion)
    }
}
