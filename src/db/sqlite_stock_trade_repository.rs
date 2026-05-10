use crate::{models::stock_trade::{AggregatedStockTrade, NewStockTrade, StockTrade}, service::portfolio_service::StockTradeRepository};
use async_trait::async_trait;

pub struct SqliteStockTradeRepository {
    db_pool: sqlx::SqlitePool
}

#[async_trait]
impl StockTradeRepository for SqliteStockTradeRepository {
    async fn get_aggregated(&self) -> Result<Vec<AggregatedStockTrade>, sqlx::Error> {
        let trades = sqlx::query_as::<_, AggregatedStockTrade>(
            "SELECT
                ticker,
                SUM(CASE WHEN trade_type = 'BUY' THEN units ELSE -units END) as total_units,
                SUM(CASE WHEN trade_type = 'BUY' THEN amount_cents ELSE -amount_cents END) as total_amount_cents,
                currency
            FROM stock_trades
            GROUP BY ticker, currency
            ORDER BY ticker"
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(trades)
    }

    async fn get_tickers(&self) -> Result<Vec<String>, sqlx::Error> {
        let tickers = sqlx::query_scalar("SELECT distinct ticker from stock_trades")
        .fetch_all(&self.db_pool)
        .await?;

        Ok(tickers)
    }
}

impl SqliteStockTradeRepository {
    pub fn new(db_pool: sqlx::SqlitePool) -> Self {
        Self { db_pool }
    }

    pub async fn get_all_trades(&self) -> Result<Vec<StockTrade>, sqlx::Error> {
        let trades = sqlx::query_as::<_, StockTrade>("SELECT * FROM stock_trades")
            .fetch_all(&self.db_pool)
            .await?;
        Ok(trades)
    }

    pub async fn save_trade(&self, trade: NewStockTrade) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO stock_trades (ticker, trade_type, trade_date, units, market_price_cents, fees_cents, amount_cents, currency) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(trade.ticker)
            .bind(trade.trade_type)
            .bind(trade.trade_date)
            .bind(trade.units)
            .bind(trade.market_price_cents)
            .bind(trade.fees_cents)
            .bind(trade.amount_cents)
            .bind(trade.currency)
            .execute(&self.db_pool)
            .await?;
        Ok(result.last_insert_rowid())
    }
}

#[cfg(test)]
mod tests {
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

    #[tokio::test]
    async fn test_get_all_trades_returns_empty() {
        let pool = setup_db().await;
        let repo = SqliteStockTradeRepository::new(pool);
        let trades = repo.get_all_trades().await.unwrap();
        assert!(trades.is_empty());
    }
    
    #[tokio::test]
    async fn test_get_all_trades_returns_records() {
        let pool = setup_db().await;
        let repo = SqliteStockTradeRepository::new(pool);
        repo.save_trade(NewStockTrade {
            ticker: "AAPL".to_string(),
            trade_type: "BUY".to_string(),
            trade_date: "2024-01-01".to_string(),
            units: 10.0,
            market_price_cents: 15000,
            fees_cents: 100,
            amount_cents: 150000,
            currency: "USD".to_string()
        }).await.unwrap();
        let trades = repo.get_all_trades().await.unwrap();

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].ticker, "AAPL");
        assert_eq!(trades[0].units, 10.0);
        assert_eq!(trades[0].amount_cents, 150000);
    }
    
    #[tokio::test]
    async fn test_get_aggregated_trades() {
        let pool = setup_db().await;
        let repo = SqliteStockTradeRepository::new(pool);
        repo.save_trade(NewStockTrade {
            ticker: "AAPL".to_string(),
            trade_type: "BUY".to_string(),
            trade_date: "2024-01-01".to_string(),
            units: 10.0,
            market_price_cents: 15000,
            fees_cents: 100,
            amount_cents: 150000,
            currency: "USD".to_string()
        }).await.unwrap();
        repo.save_trade(NewStockTrade {
            ticker: "AAPL".to_string(),
            trade_type: "SELL".to_string(),
            trade_date: "2024-01-01".to_string(),
            units: 5.0,
            market_price_cents: 16000,
            fees_cents: 100,
            amount_cents: 160000,
            currency: "USD".to_string()
        }).await.unwrap();
        repo.save_trade(NewStockTrade {
            ticker: "AAPL".to_string(),
            trade_type: "BUY".to_string(),
            trade_date: "2024-01-01".to_string(),
            units: 30.0,
            market_price_cents: 19000,
            fees_cents: 100,
            amount_cents: 190000,
            currency: "USD".to_string()
        }).await.unwrap();
        repo.save_trade(NewStockTrade {
            ticker: "GOOG".to_string(),
            trade_type: "BUY".to_string(),
            trade_date: "2024-01-01".to_string(),
            units: 10.0,
            market_price_cents: 10000,
            fees_cents: 100,
            amount_cents: 100000,
            currency: "USD".to_string()
        }).await.unwrap();
        let trades = repo.get_aggregated().await.unwrap();

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].ticker, "AAPL");
        assert_eq!(trades[0].total_units, 35.0);
        assert_eq!(trades[0].total_amount_cents, 180000);
        assert_eq!(trades[1].ticker, "GOOG");
        assert_eq!(trades[1].total_units, 10.0);
        assert_eq!(trades[1].total_amount_cents, 100000);
    }
}