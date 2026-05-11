use crate::models::{portfolio::Portfolio, portfolio::PortfolioSummary, stock_trade::AggregatedStockTrade};
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

    pub async fn get_summary(&self) -> Result<PortfolioSummary, PortfolioError> {
        let portfolio = self.get_portfolio().await?;  
        if portfolio.is_empty() {
            return Ok(PortfolioSummary::empty());
        }

        let currency = portfolio.first().map(|f: &Portfolio| f.currency.clone()).unwrap_or("AUD".to_string());
        let (total_value, total_gain_loss ) = portfolio.iter().fold(
            (0i64, 0i64),
            |acc, p| (
                acc.0 + (p.current_price_cents as f64 * p.units).round() as i64,
                acc.1 + p.gain_loss_cents
            )
        );


        Ok(PortfolioSummary { total_value_price_cents: total_value, total_gain_loss_cents: total_gain_loss, currency: currency })
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
            [("VDHG".to_string(), StockPrice{id: 1, ticker: "VDHG".to_string(), price_cents: 3623, currency: "AUD".to_string(), fetched_at: chrono::Utc::now()})].into_iter().collect()
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
            [("VDHG".to_string(), StockPrice{id: 1, ticker: "VDHG".to_string(), price_cents: 3623, currency: "AUD".to_string(), fetched_at: chrono::Utc::now()})].into_iter().collect()
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

    #[tokio::test]
    async fn test_empty_portfolio_summary() {
        let prices_repo = MockRepository::new_with_prices(
           HashMap::new()
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

        let summary = svc.get_summary().await.unwrap();

        assert_eq!(summary.total_gain_loss_cents, 0);
        assert_eq!(summary.total_value_price_cents, 0);
    }

    #[tokio::test]
    async fn test_portfolio_summary_from_all_trades() {
        let prices_repo = MockRepository::new_with_prices(
           [
            ("VDHG".to_string(), StockPrice{id: 1, ticker: "VDHG".to_string(), price_cents: 6000, currency: "AUD".to_string(), fetched_at: chrono::Utc::now()}),
            ("VAS".to_string(), StockPrice{id: 1, ticker: "VAS".to_string(), price_cents: 5000, currency: "AUD".to_string(), fetched_at: chrono::Utc::now()})
           ].into_iter().collect()
        );
        let trades_repo = MockRepository::new_with_trades(vec![AggregatedStockTrade{
            ticker: "VDHG".to_string(),
            total_units: 20.0,
            total_amount_cents: 80000,
            currency: "AUD".to_string()
        }, AggregatedStockTrade{
            ticker: "VAS".to_string(),
            total_units: 10.0,
            total_amount_cents: 60000,
            currency: "AUD".to_string()
        }]);
        let svc = PortfolioService::new(trades_repo, prices_repo);

        let summary = svc.get_summary().await.unwrap();

        assert_eq!(summary.total_gain_loss_cents, 30000);
        assert_eq!(summary.total_value_price_cents, 170000);
    }

}