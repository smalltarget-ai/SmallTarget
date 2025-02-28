use anyhow::{anyhow, Ok, Result};
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{ChatCompletionMessage, ChatCompletionRequest, ChatCompletionResponse},
    error::APIError,
};
use std::result::Result::Ok as StdOK;

/// max pixels of image to send to llm
pub const MAX_PIXELS: u32 = 1350 * 28 * 28;

pub struct OpenAiProtocalCallPayload {
    base_url: String,
    model_name: String,
    api_key: String,
    contents: Vec<ChatCompletionMessage>,
    history: Vec<ChatCompletionMessage>,
    temperature: f64,
    top_p: f64,
    max_tokens: i64,
}
impl OpenAiProtocalCallPayload {
    pub fn new(
        base_url: String,
        model_name: String,
        api_key: String,
        contents: Vec<ChatCompletionMessage>,
        history: Vec<ChatCompletionMessage>,
        temperature: f64,
        top_p: f64,
        max_tokens: i64,
    ) -> Self {
        Self {
            base_url,
            model_name,
            api_key,
            contents,
            history,
            temperature,
            top_p,
            max_tokens,
        }
    }
}

pub async fn openai_request(payload: OpenAiProtocalCallPayload) -> Result<ChatCompletionResponse> {
    let client = OpenAIClient::builder()
        .with_endpoint(payload.base_url)
        .with_api_key(payload.api_key)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build OpenAIClient: {}", e))?;

    let mut messages = Vec::new();

    messages.extend(payload.contents);
    messages.extend(payload.history);

    let req = ChatCompletionRequest::new(payload.model_name, messages)
        .temperature(payload.temperature)
        .top_p(payload.top_p)
        .max_tokens(payload.max_tokens);
    let result = match client.chat_completion(req).await {
        StdOK(response) => response,
        Err(e) => match e {
            APIError::ReqwestError(e) => {
                if let Some(status) = e.status() {
                    log::error!("HTTP request failed with status: {} - {}", status, e);
                } else {
                    log::error!("Request failed with error: {}", e);
                }
                return Err(anyhow!("API request failed: {}", e));
            }
            APIError::CustomError { message } => {
                log::error!("OpenAI API request failed: {}", message);
                return Err(anyhow!("API error: {}", message));
            }
        },
    };
    Ok(result)
}
