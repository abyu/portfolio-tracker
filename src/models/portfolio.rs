#[derive(Debug, serde::Serialize)]
pub struct Portfolio {
    pub ticker: String,
    pub units: f64,
    pub average_price_cents: i64,
    pub current_price_cents: i64,
    pub gain_loss_cents: i64,
    pub currency: String 
}