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
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct Content {
    is_success: bool,
    error: Option<Error>,
    body: Option<Vec<NewStockTrade>>,
}

#[derive(Deserialize)]
struct Error {
    reason: String,
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
                "content": "Extract trade confirmation details from these images. Return ONLY valid JSON, no other text:\n\n{\n  \"is_success\": true,\n  \"error\": null,\n  \"body\": [{\n    \"ticker\": \"string\",\n    \"trade_type\": \"BUY or SELL\",\n    \"trade_date\": \"YYYY-MM-DD\",\n    \"units\": 0.0,\n    \"market_price_cents\": integer (no decimal point),\n    \"fees_cents\": integer (no decimal point),\n    \"amount_cents\": integer (no decimal point),\n    \"currency\": \"AUD\"\n  }]\n}\n\nIf no valid trade confirmation found:\n{\n  \"is_success\": false,\n  \"error\": {\n    \"reason\": \"string\"\n  },\n  \"body\": null\n}",
                "images": &parse_request.images
            }],
            "stream": false
        });

        let response = self
            .client
            .post(chat_url)
            .json(&request)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        let value: Content = serde_json::from_str(response.message.content.trim())
            .map_err(|e| ParseError::ParseError(e.to_string()))?;

        if value.is_success {
            return Ok(value.body.unwrap_or_default());
        }

        Err(ParseError::InvalidTradeData(
            value
                .error
                .map(|e| e.reason)
                .unwrap_or("Unknown error".to_string()),
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

#[cfg(test)]
mod test {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{self, method},
    };

    #[tokio::test]
    async fn test_success_response_from_ollama() {
        let sev = MockServer::start().await;
        let inner_content = serde_json::to_string(&json!({
            "is_success": true,
            "error": null,
            "body": [{
                "ticker": "AAPL",
                "trade_type": "BUY",
                "trade_date": "2024-01-15",
                "units": 10.0,
                "market_price_cents": 15000,
                "fees_cents": 500,
                "amount_cents": 150500,
                "currency": "AUD"
            }]
        }))
        .unwrap();

        let sample_response = ResponseTemplate::new(200).set_body_json(json!({
            "model": "test-model",
            "created_at": "2024-01-15T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": inner_content
            }
        }));

        Mock::given(method("POST"))
            .and(matchers::path("/api/chat"))
            .respond_with(sample_response)
            .mount(&sev)
            .await;
        let cli = Ollama::new(reqwest::Client::new(), sev.uri(), "model".to_string());

        let resp = cli
            .parse_trade_confirmation(LLMExtractTradeTransactionRequest { images: vec![] })
            .await;

        // assert!(resp.is_ok());
        assert_eq!(resp.unwrap().len(), 1)
    }

    #[tokio::test]
    async fn test_invalid_response_from_ollama() {
        let sev = MockServer::start().await;

        let sample_response = ResponseTemplate::new(200).set_body_json(json!({
            "model": "test-model",
            "created_at": "2024-01-15T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": "THIS SHOULD HAVE BEEN A VALID JSON"
            }
        }));

        Mock::given(method("POST"))
            .and(matchers::path("/api/chat"))
            .respond_with(sample_response)
            .mount(&sev)
            .await;
        let cli = Ollama::new(reqwest::Client::new(), sev.uri(), "model".to_string());

        let resp = cli
            .parse_trade_confirmation(LLMExtractTradeTransactionRequest { images: vec![] })
            .await;

        assert!(resp.is_err_and(|e| matches!(e, ParseError::ParseError(_))));
    }

    #[tokio::test]
    async fn test_no_trade_data_response_from_ollama() {
        let sev = MockServer::start().await;

        let inner_content = serde_json::to_string(&json!({
            "is_success": false,
            "error": {
                "reason": "NO TRADE DATA_FOUND"
            }
        }))
        .unwrap();

        let sample_response = ResponseTemplate::new(200).set_body_json(json!({
            "model": "test-model",
            "created_at": "2024-01-15T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": inner_content
            }
        }));

        Mock::given(method("POST"))
            .and(matchers::path("/api/chat"))
            .respond_with(sample_response)
            .mount(&sev)
            .await;
        let cli = Ollama::new(reqwest::Client::new(), sev.uri(), "model".to_string());

        let resp = cli
            .parse_trade_confirmation(LLMExtractTradeTransactionRequest { images: vec![] })
            .await;

        assert!(resp.is_err_and(
            |e| matches!(e, ParseError::InvalidTradeData(msg) if msg == "NO TRADE DATA_FOUND")
        ));
    }

    #[tokio::test]
    async fn test_missing_response_from_ollama() {
        let sev = MockServer::start().await;

        let cli = Ollama::new(reqwest::Client::new(), sev.uri(), "model".to_string());

        let resp = cli
            .parse_trade_confirmation(LLMExtractTradeTransactionRequest { images: vec![] })
            .await;

        assert!(resp.is_err_and(|e| matches!(e, ParseError::HttpError(_))));
    }

    #[tokio::test]
    async fn test_empty_success_response_from_ollama() {
        let sev = MockServer::start().await;

        let inner_content = serde_json::to_string(&json!({
            "is_success": true,
        }))
        .unwrap();

        let sample_response = ResponseTemplate::new(200).set_body_json(json!({
            "model": "test-model",
            "created_at": "2024-01-15T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": inner_content
            }
        }));

        Mock::given(method("POST"))
            .and(matchers::path("/api/chat"))
            .respond_with(sample_response)
            .mount(&sev)
            .await;
        let cli = Ollama::new(reqwest::Client::new(), sev.uri(), "model".to_string());

        let resp = cli
            .parse_trade_confirmation(LLMExtractTradeTransactionRequest { images: vec![] })
            .await;

        assert!(resp.is_ok_and(|f| f.is_empty()));
    }

    #[tokio::test]
    async fn test_failure_with_missing_error_response_from_ollama() {
        let sev = MockServer::start().await;

        let inner_content = serde_json::to_string(&json!({
            "is_success": false,
        }))
        .unwrap();

        let sample_response = ResponseTemplate::new(200).set_body_json(json!({
            "model": "test-model",
            "created_at": "2024-01-15T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": inner_content
            }
        }));

        Mock::given(method("POST"))
            .and(matchers::path("/api/chat"))
            .respond_with(sample_response)
            .mount(&sev)
            .await;
        let cli = Ollama::new(reqwest::Client::new(), sev.uri(), "model".to_string());

        let resp = cli
            .parse_trade_confirmation(LLMExtractTradeTransactionRequest { images: vec![] })
            .await;

        assert!(resp.is_err_and(
            |f| matches!(f, ParseError::InvalidTradeData(msg) if msg == "Unknown error")
        ));
    }

    #[tokio::test]
    async fn test_request_body_contains_model_and_images() {
        let sev = MockServer::start().await;

        let inner_content = serde_json::to_string(&json!({
            "is_success": true,
            "error": null,
            "body": []
        }))
        .unwrap();

        let sample_response = ResponseTemplate::new(200).set_body_json(json!({
            "model": "test-model",
            "created_at": "2024-01-15T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": inner_content
            }
        }));

        Mock::given(method("POST"))
            .and(matchers::path("/api/chat"))
            .and(matchers::body_partial_json(json!({
                "model": "test-model-check",
                "messages": [{
                    "images": ["img-data-1", "img-data-2"]
                }]
            })))
            .respond_with(sample_response)
            .mount(&sev)
            .await;

        let cli = Ollama::new(
            reqwest::Client::new(),
            sev.uri(),
            "test-model-check".to_string(),
        );

        let resp = cli
            .parse_trade_confirmation(LLMExtractTradeTransactionRequest {
                images: vec!["img-data-1".to_string(), "img-data-2".to_string()],
            })
            .await;

        assert!(resp.is_ok());
    }
}
