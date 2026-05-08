use crate::models::stock_price::*;

pub struct StockPriceRepository {
    pool: sqlx::SqlitePool,
}

impl StockPriceRepository {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_by_ticker(&self, ticker: &str) -> Result<Option<StockPrice>, sqlx::Error> {
        let price = sqlx::query_as::<_, StockPrice>("SELECT * FROM stock_prices WHERE ticker = ?")
            .bind(ticker)
            .fetch_optional(&self.pool)
            .await?;
        Ok(price)
    }

    pub async fn upsert_price(&self, price: NewStockPrice) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO stock_prices (ticker, price_cents, currency) VALUES (?, ?, ?)
                ON CONFLICT(ticker) DO UPDATE SET 
                price_cents = excluded.price_cents,
                fetched_at = datetime('now')"
            )
            .bind(price.ticker)
            .bind(price.price_cents)
            .bind(price.currency)
            .execute(&self.pool)
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
    async fn test_get_ticker_price_for_missing_ticker_returns_none() {
        let pool = setup_db().await;
        let repo = StockPriceRepository::new(pool);
        let price = repo.get_by_ticker("VDHG").await.unwrap();
        assert!(price.is_none());
    }
    
    #[tokio::test]
    async fn test_get_ticker_price_for_a_ticker() {
        let pool = setup_db().await;
        let repo = StockPriceRepository::new(pool);
        let _ = repo.upsert_price(NewStockPrice { ticker: "VDHG".to_string(), price_cents: 3422, currency: "AUD".to_string()}).await;
        let price = repo.get_by_ticker("VDHG").await.unwrap();
        assert!(price.is_some_and(|v| v.price_cents == 3422 && v.ticker == "VDHG"));
    }

    #[tokio::test]
    async fn test_upsert_when_price_for_ticker_exists() {
        let pool = setup_db().await;
        let repo = StockPriceRepository::new(pool);
        let _ = repo.upsert_price(NewStockPrice { ticker: "VDHG".to_string(), price_cents: 3422, currency: "AUD".to_string()}).await;
        let old_price = repo.get_by_ticker("VDHG").await.unwrap();
        
        let _ = repo.upsert_price(NewStockPrice { ticker: "VDHG".to_string(), price_cents: 3522, currency: "AUD".to_string()}).await;
        let new_price = repo.get_by_ticker("VDHG").await.unwrap();

        assert!(old_price.is_some_and(|v| v.price_cents == 3422 && v.ticker == "VDHG"));
        assert!(new_price.is_some_and(|v| v.price_cents == 3522 && v.ticker == "VDHG"));
    }
}