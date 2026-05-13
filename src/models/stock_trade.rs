#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct StockTrade {
    pub id: i64,
    pub ticker: String,
    pub trade_type: String,
    pub trade_date: String,
    pub units: f64,
    pub market_price_cents: i64,
    pub fees_cents: i64,
    pub amount_cents: i64,
    pub currency: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub user_id: i64
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct AggregatedStockTrade {
    pub ticker: String,
    pub total_units: f64,
    pub total_amount_cents: i64,
    pub currency: String,
}

#[derive(Debug)]
pub struct NewStockTrade {
    pub ticker: String,
    pub trade_type: String,
    pub trade_date: String,
    pub units: f64,
    pub market_price_cents: i64,
    pub fees_cents: i64,
    pub amount_cents: i64,
    pub currency: String,
}