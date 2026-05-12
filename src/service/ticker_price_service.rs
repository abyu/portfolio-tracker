use crate::models::stock_price::{NewStockPrice, StockPrice};
use crate::service::price_service::{PriceError, PriceService};
use async_trait::async_trait;
use chrono::Utc;
use yahoo_finance_api::YahooConnector;
pub struct TickerPriceService<T: TickerPriceProvider, S: StockPriceRepository> {
    stock_price_repo: S,
    price_provider: T,
}

#[async_trait]
pub trait StockPriceRepository: Send + Sync {
    async fn upsert_price(&self, price: NewStockPrice) -> Result<i64, sqlx::Error>;
    async fn get_by_ticker(&self, ticker: &str) -> Result<Option<StockPrice>, sqlx::Error>;
}

#[async_trait]
pub trait TickerPriceProvider: Send + Sync {
    async fn get_price(&self, ticker: &str) -> Result<f64, PriceError>;
}

impl<T: TickerPriceProvider, S: StockPriceRepository> TickerPriceService<T, S> {
    pub fn new(repo: S, api: T) -> Self {
        Self {
            stock_price_repo: repo,
            price_provider: api,
        }
    }
}

#[async_trait]
impl<T: TickerPriceProvider, S: StockPriceRepository> PriceService for TickerPriceService<T, S> {
    async fn get_by_ticker(&self, ticker: &str) -> Result<Option<StockPrice>, PriceError> {
        self.update_prices(vec![ticker.to_string()])
            .await
            .map_err(|op| PriceError::APIError(op.to_string()))?;

        Ok(self.stock_price_repo.get_by_ticker(ticker).await?)
    }

    async fn update_prices(&self, tickers: Vec<String>) -> Result<(), PriceError> {
        for ticker in tickers {
            let is_fresh = self
                .stock_price_repo
                .get_by_ticker(&ticker)
                .await?
                .filter(|p| {
                    Utc::now().signed_duration_since(p.fetched_at) <= chrono::Duration::minutes(20)
                })
                .is_some();

            if !is_fresh {
                let price = self.price_provider.get_price(ticker.as_str()).await?;
                let ticker_price = (price * 100.0).round() as i64;
                self.stock_price_repo
                    .upsert_price(NewStockPrice {
                        ticker: ticker,
                        price_cents: ticker_price,
                        currency: "AUD".to_string(),
                    })
                    .await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TickerPriceProvider for YahooConnector {
    async fn get_price(&self, ticker: &str) -> Result<f64, PriceError> {
        let yahoo_ticker = format!("{}.AX", ticker);
        let quotes = self.get_latest_quotes(&yahoo_ticker, "1d").await?;
        let price = quotes.last_quote()?;
        Ok(price.close)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    struct MockPriceProvider {
        price: f64,
    }

    struct MockStockPriceRepository {
        written: std::sync::Mutex<Vec<NewStockPrice>>,
        existing: HashMap<String, StockPrice>,
    }

    #[async_trait]
    impl TickerPriceProvider for MockPriceProvider {
        async fn get_price(&self, ticker: &str) -> Result<f64, PriceError> {
            Ok(self.price)
        }
    }

    impl MockStockPriceRepository {
        fn new(existing: HashMap<String, StockPrice>) -> Self {
            Self {
                written: std::sync::Mutex::new(vec![]),
                existing: existing,
            }
        }
    }
    #[async_trait]
    impl StockPriceRepository for MockStockPriceRepository {
        async fn upsert_price(&self, price: NewStockPrice) -> Result<i64, sqlx::Error> {
            self.written.lock().unwrap().push(price);
            Ok(1)
        }

        async fn get_by_ticker(&self, ticker: &str) -> Result<Option<StockPrice>, sqlx::Error> {
            Ok(self.existing.get(ticker).cloned())
        }
    }

    #[tokio::test]
    async fn test_update_price_for_ticker_and_persist() {
        let price_provider = MockPriceProvider { price: 34.23 };
        let repo = MockStockPriceRepository::new(HashMap::new());
        let svc = TickerPriceService::new(repo, price_provider);

        svc.update_prices(vec!["VDHG".to_string()]).await.unwrap();

        let price = svc.stock_price_repo.written.lock().unwrap();

        assert_eq!(price.len(), 1);
        assert_eq!(price[0].price_cents, 3423)
    }

    #[tokio::test]
    async fn test_update_price_skips_for_ticker_with_fresh_prices() {
        let price_provider = MockPriceProvider { price: 34.23 };
        let repo =
            MockStockPriceRepository::new([("VDHG".to_string(), new_stock_price("VDHG".to_string(), 3423))].into_iter().collect());
        let svc = TickerPriceService::new(repo, price_provider);

        svc.update_prices(vec!["VDHG".to_string()]).await.unwrap();

        let price = svc.stock_price_repo.written.lock().unwrap();

        assert_eq!(price.len(), 0);
    }

    fn new_stock_price(ticker: String, price: i64) -> StockPrice{
        StockPrice{
            id: 0,
            ticker: ticker.clone(),
            price_cents: price,
            currency: "AUD".to_string(),
            fetched_at: Utc::now()
        }
    }
}
