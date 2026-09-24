#[derive(Debug, sqlx::FromRow, serde::Serialize, Clone)]
pub struct StockPrice {
    pub id: i64,
    pub ticker: String,
    pub price_cents: i64,
    pub currency: String,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct NewStockPrice {
    pub ticker: String,
    pub price_cents: i64,
    pub currency: String,
}
