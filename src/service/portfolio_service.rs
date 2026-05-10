use std::vec;

use crate::db::{sqlite_stock_price_repository::SqliteStockPriceRepository, sqlite_stock_trade_repository::SqliteStockTradeRepository};
use crate::models::portfolio::Portfolio;

pub struct PortfolioService{
    stock_trades_repo: SqliteStockTradeRepository,
    stock_prices_repo: SqliteStockPriceRepository
}

impl PortfolioService {
    pub fn new(stock_trades: SqliteStockTradeRepository, stock_prices: SqliteStockPriceRepository) -> Self {
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