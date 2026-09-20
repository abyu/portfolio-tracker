use crate::models::{llm_request::LLMExtractTradeTransactionRequest, stock_trade::NewStockTrade};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

pub struct Ollama {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    model: String,
    created_at: String,
    message: Message 
}

#[derive(Deserialize)]
struct Message {
    role: String,
    content: String
}

#[derive(Deserialize)]
struct Content {
    is_success: bool,
    error: Option<Error>,
    body: Option<Vec<NewStockTrade>>
}

#[derive(Deserialize)]
struct Error {
    code: String,
    message: String
}

#[async_trait]
pub trait OllamaHttpClient: Send + Sync {
    async fn parse_trade_confirmation(
        &self,
        parse_request: LLMExtractTradeTransactionRequest,
    ) -> Result<Vec<NewStockTrade>, ParseError>;
}

impl Ollama {
    pub fn new(client: reqwest::Client, base_url: String, model: String) -> Self {
        Self {
            client,
            base_url,
            model,
        }
    }
}

#[async_trait]
impl OllamaHttpClient for Ollama {
    async fn parse_trade_confirmation(
        &self,
        parse_request: LLMExtractTradeTransactionRequest,
    ) -> Result<Vec<NewStockTrade>, ParseError> {
        let chat_url = format!("{0}/api/chat", self.base_url);
        let request = json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": "Extract trade confirmation details from these images. Return ONLY valid JSON, no other text:\n\n{\n  \"is_success\": true,\n  \"error\": null,\n  \"body\": [{\n    \"ticker\": \"string\",\n    \"trade_type\": \"BUY or SELL\",\n    \"trade_date\": \"YYYY-MM-DD\",\n    \"units\": 0.0,\n    \"market_price_cents\": integer (no decimal point),\n    \"fees_cents\": integer (no decimal point),\n    \"amount_cents\": integer (no decimal point),\n    \"currency\": \"AUD\"\n  }]\n}\n\nIf no valid trade confirmation found:\n{\n  \"is_success\": false,\n  \"error\": \"reason\",\n  \"body\": null\n}",
                "images": &parse_request.images
            }],
            "stream": false
        });

        let response = self.client.post(chat_url).json(&request).send().await?
        .json::<OllamaResponse>()
        .await?;

        let value : Content = serde_json::from_str(response.message.content.trim()).map_err(|e| ParseError::ParseError(e.to_string()))?;

        if value.is_success {
            return Ok(value.body.unwrap_or_default());
        }

        Err(ParseError::InvalidTradeData(
            value.error.map(|e| e.message).unwrap_or("Unknown error".to_string())
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Failed to parse LLM response: {0}")]
    ParseError(String),
    #[error("LLM returned invalid trade data: {0}")]
    InvalidTradeData(String),
}
