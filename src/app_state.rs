use std::sync::Arc;

use sqlx::PgPool;

use crate::service::{jwt_service::JwtService, price_service::PriceService, user_service::UserServiceTrait};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub price_service: Arc<dyn PriceService>,
    pub jwt_service: Arc<JwtService>,
    pub user_service: Arc<dyn UserServiceTrait>
}