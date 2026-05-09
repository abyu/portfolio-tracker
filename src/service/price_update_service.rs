use crate::{db::stock_price_repository::StockPriceRepository, models::stock_price::NewStockPrice};
use yahoo_finance_api::{YahooConnector, YahooError};
use async_trait::async_trait;
pub struct PriceUpdateService<T: TickerPriceProvider> {
    stock_price_repo: StockPriceRepository,
    price_provider: T
}

#[async_trait]
pub trait TickerPriceProvider {
    async fn get_price(&self, ticker: &str) -> Result<f64, PriceUpdateError>;
}

impl<T: TickerPriceProvider> PriceUpdateService<T> {
    pub fn new(repo: StockPriceRepository, api: T) -> Self {
        Self { stock_price_repo:repo, price_provider: api }
    }

    pub async fn update_prices(&self, tickers: Vec<String>) -> Result<(), PriceUpdateError> {
        for ticker in tickers {
            let price = self.price_provider.get_price(ticker.as_str()).await?;
            let ticker_price = (price * 100.0).round() as i64;
            self.stock_price_repo.upsert_price(
                NewStockPrice{
                    ticker : ticker,
                    price_cents: ticker_price,
                    currency: "AUD".to_string(),
                }
            ).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl TickerPriceProvider for YahooConnector {
    async fn get_price(&self, ticker: &str) -> Result<f64, PriceUpdateError> {
        let yahoo_ticker = format!("{}.AX", ticker);
        let quotes = self.get_latest_quotes(&yahoo_ticker, "1d").await?;
        let price = quotes.last_quote()?;
        Ok(price.close)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PriceUpdateError {
     #[error("Yahoo API error: {0}")]
    APIError(#[from] YahooError),
    #[error("DB error: {0}")]
    DBError(#[from] sqlx::Error)
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

        pool
    }
    struct MockPriceProvider {
        price: f64
    }
    
    #[async_trait]
    impl TickerPriceProvider for MockPriceProvider {
        async fn get_price(&self, ticker: &str) -> Result<f64, PriceUpdateError> {
            Ok(self.price)
        }
    }

    #[tokio::test]
    async fn test_update_price_for_ticker_and_persist() {
        let price_provider = MockPriceProvider { price: 34.23 };
        let pool = setup_db().await;
        let repo = StockPriceRepository::new(pool.clone());
        let svc = PriceUpdateService::new(StockPriceRepository::new(pool), price_provider);
        
        svc.update_prices(vec!["VDHG".to_string()]).await.unwrap();
        
        let price = repo.get_by_ticker("VDHG").await.unwrap();

        assert!(price.is_some_and(|v| v.price_cents == 3423))
    }
}