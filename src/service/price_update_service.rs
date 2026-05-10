use crate::models::stock_price::NewStockPrice;
use yahoo_finance_api::{YahooConnector, YahooError};
use async_trait::async_trait;
pub struct PriceUpdateService<T: TickerPriceProvider, S: StockPriceRepository> {
    stock_price_repo: S,
    price_provider: T
}

#[async_trait]
pub trait StockPriceRepository {
    async fn upsert_price(&self, price: NewStockPrice) -> Result<i64, sqlx::Error>;
}

#[async_trait]
pub trait TickerPriceProvider {
    async fn get_price(&self, ticker: &str) -> Result<f64, PriceUpdateError>;
}

impl<T: TickerPriceProvider, S: StockPriceRepository> PriceUpdateService<T, S> {
    pub fn new(repo: S, api: T) -> Self {
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
    struct MockPriceProvider {
        price: f64
    }

    struct MockStockPriceRepository {
        written: std::sync::Mutex<Vec<NewStockPrice>>
    }
    
    #[async_trait]
    impl TickerPriceProvider for MockPriceProvider {
        async fn get_price(&self, ticker: &str) -> Result<f64, PriceUpdateError> {
            Ok(self.price)
        }
    }

    impl MockStockPriceRepository {
        fn new() -> Self {
            Self { written: std::sync::Mutex::new(vec![]) }
        }
    }
    #[async_trait]
    impl StockPriceRepository for MockStockPriceRepository {
        async fn upsert_price(&self,price: NewStockPrice) ->  Result<i64,sqlx::Error> {
            self.written.lock().unwrap().push(price);
            Ok(1)
        }
    }

    #[tokio::test]
    async fn test_update_price_for_ticker_and_persist() {
        let price_provider = MockPriceProvider { price: 34.23 };
        let repo = MockStockPriceRepository::new();
        let svc = PriceUpdateService::new(repo, price_provider);
        
        svc.update_prices(vec!["VDHG".to_string()]).await.unwrap();
        
        let price = svc.stock_price_repo.written.lock().unwrap();

        assert_eq!(price.len(), 1);
        assert_eq!(price[0].price_cents, 3423)
    }
}