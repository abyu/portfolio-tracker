use axum::async_trait;
use thiserror::Error;
use yahoo_finance_api::YahooError;

use crate::models::stock_price::StockPrice;

#[async_trait]
pub trait PriceService: Send + Sync {
    async fn get_by_ticker(&self, ticker: &str) -> Result<Option<StockPrice>, PriceError>;
    async fn update_prices(&self, tickers: Vec<String>) -> Result<(), PriceError>;
}

#[derive(Debug, Error)]
pub enum PriceError {
    #[error("Price DB error: {0}")]
    DBError(#[from] sqlx::Error),
    #[error("Price API error")]
    APIError(String),
    #[error("Yahoo API error: {0}")]
    YahooAPIError(#[from] YahooError),
}
