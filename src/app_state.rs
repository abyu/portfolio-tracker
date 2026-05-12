use std::sync::Arc;

use crate::{config::AppConfig, service::portfolio_service::PortfolioServiceTrait};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: AppConfig,
    pub tera: tera::Tera,
    pub portfolio_service: Arc<dyn PortfolioServiceTrait + Send + Sync>
}