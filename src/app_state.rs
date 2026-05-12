use std::sync::Arc;

use crate::service::portfolio_service::PortfolioServiceTrait;

#[derive(Clone)]
pub struct AppState {
    pub portfolio_service: Arc<dyn PortfolioServiceTrait + Send + Sync>
}