use crate::{models::stock_price::*, service::price_update_service::StockPriceRepository};
use async_trait::async_trait;

pub struct PostgresStockPriceRepository {
    pool: sqlx::PgPool,
}


#[async_trait]
impl StockPriceRepository for PostgresStockPriceRepository {
    async fn upsert_price(&self, price: NewStockPrice) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar(
            "INSERT INTO stock_prices (ticker, price_cents, currency) VALUES ($1, $2, $3)
                ON CONFLICT(ticker) DO UPDATE SET 
                price_cents = excluded.price_cents,
                fetched_at = NOW()
                RETURNING id"
            )
            .bind(price.ticker)
            .bind(price.price_cents)
            .bind(price.currency)
            .fetch_one(&self.pool)
            .await?;
        Ok(result)
    }

    async fn get_by_ticker(&self, ticker: &str) -> Result<Option<StockPrice>, sqlx::Error> {
        let price = sqlx::query_as::<_, StockPrice>("SELECT * FROM stock_prices WHERE ticker = $1")
            .bind(ticker)
            .fetch_optional(&self.pool)
            .await?;
        Ok(price)
    }
}

impl PostgresStockPriceRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_ticker_price_for_missing_ticker_returns_none(pool :sqlx::PgPool) {
        let repo = PostgresStockPriceRepository::new(pool);
        let price = repo.get_by_ticker("VDHG").await.unwrap();
        assert!(price.is_none());
    }
    
    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_ticker_price_for_a_ticker(pool :sqlx::PgPool) {
        let repo = PostgresStockPriceRepository::new(pool);
        let _ = repo.upsert_price(NewStockPrice { ticker: "VDHG".to_string(), price_cents: 3422, currency: "AUD".to_string()}).await;
        let price = repo.get_by_ticker("VDHG").await.unwrap();
        assert!(price.is_some_and(|v| v.price_cents == 3422 && v.ticker == "VDHG"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_upsert_when_price_for_ticker_exists(pool :sqlx::PgPool) {
        let repo = PostgresStockPriceRepository::new(pool);
        let _ = repo.upsert_price(NewStockPrice { ticker: "VDHG".to_string(), price_cents: 3422, currency: "AUD".to_string()}).await;
        let old_price = repo.get_by_ticker("VDHG").await.unwrap();
        
        let _ = repo.upsert_price(NewStockPrice { ticker: "VDHG".to_string(), price_cents: 3522, currency: "AUD".to_string()}).await;
        let new_price = repo.get_by_ticker("VDHG").await.unwrap();

        assert!(old_price.is_some_and(|v| v.price_cents == 3422 && v.ticker == "VDHG"));
        assert!(new_price.is_some_and(|v| v.price_cents == 3522 && v.ticker == "VDHG"));
    }
}