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
pub trait OllamaHttpClient {
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
                "content": "Extract the following fields ticker, quantity, price, total amount, fees, date across all these  trade confirmation images, skip any images if it not a valid trade confirmation file. Return response in this format Return ONLY valid JSON in this exact format, no other text:{is_success: true, error: null, body: [{ticker: String,trade_type: BUY | SELL,trade_date: String in format YYYY-MM-DD,units: f64,market_price_cents: i64,fees_cents: i64,amount_cents: i64,currency: String,}]}If none of them are a trade confirmation return:{is_success: false,error: reason here,body: null}",
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
