use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct Portfolio {
    pub ticker: String,
    pub units: f64,
    pub average_price_cents: i64,
    pub current_price_cents: i64,
    pub gain_loss_cents: i64,
    pub currency: String 
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PortfolioSummary {
    pub total_value_price_cents: i64,
    pub total_gain_loss_cents: i64,
    pub currency: String
}

impl PortfolioSummary {
    pub fn empty() -> Self {
        Self {
            total_value_price_cents: 0,
            total_gain_loss_cents: 0,
            currency: "AUD".to_string(),
        }
    }
}