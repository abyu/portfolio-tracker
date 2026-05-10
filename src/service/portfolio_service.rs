use crate::models::{portfolio::Portfolio, stock_trade::AggregatedStockTrade};
use crate::service::price_update_service::StockPriceRepository;
use async_trait::async_trait;

pub struct PortfolioService<S: StockPriceRepository, P: StockTradeRepository>{
    stock_trades_repo: P,
    stock_prices_repo: S
}

#[async_trait]
pub trait StockTradeRepository {
    async fn get_aggregated(&self) -> Result<Vec<AggregatedStockTrade>, sqlx::Error>;
    async fn get_tickers(&self) -> Result<Vec<String>, sqlx::Error>;
}

impl<S: StockPriceRepository, P: StockTradeRepository> PortfolioService<S, P> {
    pub fn new(stock_trades: P, stock_prices: S) -> Self {
        Self { stock_trades_repo: stock_trades, stock_prices_repo: stock_prices }
    }

    pub async  fn get_portfolio(&self) -> Result<Vec<Portfolio>, PortfolioError> {
        let trades = self.stock_trades_repo.get_aggregated().await?;
        let mut portfolios = Vec::new();
    
        for trade in trades {
            if let Some(price) = self.stock_prices_repo.get_by_ticker(&trade.ticker).await? {
                let total_value_cents = (trade.total_units * price.price_cents as f64).round() as i64;
                portfolios.push(Portfolio {
                    ticker: trade.ticker,
                    units: trade.total_units,
                    average_price_cents: (trade.total_amount_cents as f64 / trade.total_units).round() as i64,
                    current_price_cents: price.price_cents,
                    gain_loss_cents: total_value_cents - trade.total_amount_cents,
                    currency: trade.currency,
                });
            }
        }
        Ok(portfolios)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PortfolioError{
    #[error("Portfolio DB error: {0}")]
    DBError(#[from] sqlx::Error),
}

#[cfg(test)]
mod test{
    use std::collections::HashMap;
    use super::*;
    use crate::models::stock_price::{NewStockPrice, StockPrice};

    struct MockRepository {
        trades: Vec<AggregatedStockTrade>,
        prices: HashMap<String, StockPrice>
    }

    impl MockRepository {
        fn new_with_trades(trades: Vec<AggregatedStockTrade>) -> Self {
            Self { trades: trades, prices: HashMap::new() }
        }
        
        fn new_with_prices(prices: HashMap<String, StockPrice>) -> Self {
            Self {  trades: vec![], prices }
        }
    }

    #[async_trait]
    impl StockPriceRepository for MockRepository {
        async fn upsert_price(&self, price: NewStockPrice) -> Result<i64, sqlx::Error> {
            Ok(1)
        }
        async fn get_by_ticker(&self, ticker: &str) -> Result<Option<StockPrice>, sqlx::Error> {
            Ok(self.prices.get(ticker).cloned())
        }
    }

    #[async_trait]
    impl StockTradeRepository for MockRepository {
        async fn get_aggregated(&self) -> Result<Vec<AggregatedStockTrade>, sqlx::Error> {
            Ok(self.trades.clone())
        }

        async fn get_tickers(&self) -> Result<Vec<String>, sqlx::Error> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_build_portfolio_based_on_ticker_current_price() {
        let prices_repo = MockRepository::new_with_prices(
            [("VDHG".to_string(), StockPrice{id: 1, ticker: "VDHG".to_string(), price_cents: 3623, currency: "AUD".to_string(), fetched_at:"".to_string()})].into_iter().collect()
        );
        let trades_repo = MockRepository::new_with_trades(vec![AggregatedStockTrade{
            ticker: "VDHG".to_string(),
            total_units: 20.0,
            total_amount_cents: 68460,
            currency: "AUD".to_string()
        }]);
        let svc = PortfolioService::new(trades_repo, prices_repo);

        let portfolio = svc.get_portfolio().await.unwrap();

        assert_eq!(portfolio.len(), 1);
        assert_eq!(portfolio[0].average_price_cents, 3423);
        assert_eq!(portfolio[0].gain_loss_cents, 4000);
        assert_eq!(portfolio[0].current_price_cents, 3623);
        assert_eq!(portfolio[0].ticker, "VDHG");
    }

    #[tokio::test]
    async fn test_build_portfolio_skips_tickers_with_no_current_price() {
        let prices_repo = MockRepository::new_with_prices(
            [("VDHG".to_string(), StockPrice{id: 1, ticker: "VDHG".to_string(), price_cents: 3623, currency: "AUD".to_string(), fetched_at:"".to_string()})].into_iter().collect()
        );
        let trades_repo = MockRepository::new_with_trades(vec![AggregatedStockTrade{
            ticker: "VDHG".to_string(),
            total_units: 20.0,
            total_amount_cents: 68460,
            currency: "AUD".to_string()
        }, AggregatedStockTrade{
            ticker: "VAS".to_string(),
            total_units: 20.0,
            total_amount_cents: 68460,
            currency: "AUD".to_string()
        }]);
        let svc = PortfolioService::new(trades_repo, prices_repo);

        let portfolio = svc.get_portfolio().await.unwrap();

        assert_eq!(portfolio.len(), 1);
        assert_eq!(portfolio[0].average_price_cents, 3423);
        assert_eq!(portfolio[0].gain_loss_cents, 4000);
        assert_eq!(portfolio[0].current_price_cents, 3623);
        assert_eq!(portfolio[0].ticker, "VDHG");
    }

}