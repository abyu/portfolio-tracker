use std::sync::Arc;

use sqlx::PgPool;

use crate::service::price_service::PriceService;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub price_service: Arc<dyn PriceService>,
}