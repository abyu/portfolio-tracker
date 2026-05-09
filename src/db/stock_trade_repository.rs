use crate::models::stock_trade::StockTrade;
use crate::models::stock_trade::NewStockTrade;

pub struct StockTradeRepository {
    db_pool: sqlx::SqlitePool
}

impl StockTradeRepository {
    pub fn new(db_pool: sqlx::SqlitePool) -> Self {
        Self { db_pool }
    }

    pub async fn get_all_trades(&self) -> Result<Vec<StockTrade>, sqlx::Error> {
        let trades = sqlx::query_as::<_, StockTrade>("SELECT * FROM stock_trades")
            .fetch_all(&self.db_pool)
            .await?;
        Ok(trades)
    }

    pub async fn get_tickers(&self) -> Result<Vec<String>, sqlx::Error> {
        let tickers = sqlx::query_scalar("SELECT distinct ticker from stock_trades")
        .fetch_all(&self.db_pool)
        .await?;

        Ok(tickers)
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
        let repo = StockTradeRepository::new(pool);
        let trades = repo.get_all_trades().await.unwrap();
        assert!(trades.is_empty());
    }
    
    #[tokio::test]
    async fn test_get_all_trades_returns_records() {
        let pool = setup_db().await;
        let repo = StockTradeRepository::new(pool);
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
}