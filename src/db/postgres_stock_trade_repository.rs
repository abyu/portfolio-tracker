use crate::tasks::price_update_task::StockTradeRepository;

pub struct PostgresStockTradeRepository {
    db_pool: sqlx::PgPool,
}

impl PostgresStockTradeRepository {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl StockTradeRepository for PostgresStockTradeRepository {
    async fn get_tickers(&self) -> Result<Vec<String>, sqlx::Error> {
        let tickers = sqlx::query_scalar("SELECT distinct ticker from stock_trades")
            .fetch_all(&self.db_pool)
            .await?;

        Ok(tickers)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::portfolio_service::UserStockTradeRepository;
    use crate::{
        db::postgres_user_stock_trade_repository::PostgresUserStockTradeRepository,
        models::stock_trade::NewStockTrade,
    };

    async fn create_test_user(pool: &sqlx::PgPool, email: String) -> i64 {
        sqlx::query_scalar("INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id")
            .bind(email)
            .bind("Test User")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_tickers_returns_empty(pool: sqlx::PgPool) {
        let repo = PostgresStockTradeRepository::new(pool.clone());
        let trades = repo.get_tickers().await.unwrap();
        assert!(trades.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_tickers_returns_all_unique_tickers(pool: sqlx::PgPool) {
        let user1 = create_test_user(&pool, "test1@email.com".to_string()).await;
        let user2 = create_test_user(&pool, "test2@email.com".to_string()).await;
        let repo1 = PostgresUserStockTradeRepository::new(pool.clone(), user1);
        repo1
            .save_trade(NewStockTrade {
                ticker: "AAPL".to_string(),
                trade_type: "BUY".to_string(),
                trade_date: "2024-01-01".to_string(),
                units: 10.0,
                market_price_cents: 15000,
                fees_cents: 100,
                amount_cents: 150000,
                currency: "USD".to_string(),
            })
            .await
            .unwrap();
        repo1
            .save_trade(NewStockTrade {
                ticker: "AAPL".to_string(),
                trade_type: "BUY".to_string(),
                trade_date: "2024-03-01".to_string(),
                units: 10.0,
                market_price_cents: 16000,
                fees_cents: 100,
                amount_cents: 160000,
                currency: "USD".to_string(),
            })
            .await
            .unwrap();
        repo1
            .save_trade(NewStockTrade {
                ticker: "GOOG".to_string(),
                trade_type: "BUY".to_string(),
                trade_date: "2024-01-01".to_string(),
                units: 10.0,
                market_price_cents: 15000,
                fees_cents: 100,
                amount_cents: 150000,
                currency: "USD".to_string(),
            })
            .await
            .unwrap();
        let repo2 = PostgresUserStockTradeRepository::new(pool.clone(), user2);
        repo2
            .save_trade(NewStockTrade {
                ticker: "AAPL".to_string(),
                trade_type: "BUY".to_string(),
                trade_date: "2024-01-01".to_string(),
                units: 10.0,
                market_price_cents: 15000,
                fees_cents: 100,
                amount_cents: 150000,
                currency: "USD".to_string(),
            })
            .await
            .unwrap();
        repo2
            .save_trade(NewStockTrade {
                ticker: "XYZ".to_string(),
                trade_type: "BUY".to_string(),
                trade_date: "2024-01-01".to_string(),
                units: 10.0,
                market_price_cents: 15000,
                fees_cents: 100,
                amount_cents: 150000,
                currency: "USD".to_string(),
            })
            .await
            .unwrap();

        let repo = PostgresStockTradeRepository::new(pool.clone());
        let mut tickers = repo.get_tickers().await.unwrap();
        tickers.sort();
        assert_eq!(tickers.len(), 3);
        assert_eq!(tickers, vec!["AAPL", "GOOG", "XYZ"]);
    }
}
