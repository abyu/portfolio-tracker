use std::sync::Arc;

use crate::db::postgres_stock_trade_repository::PostgresStockTradeRepository;
use crate::models::portfolio::{Portfolio, PortfolioSummary};
use crate::app_state::AppState;
use crate::service::portfolio_service::{PortfolioService, PortfolioServiceTrait};
use crate::service::price_service::PriceService;
use axum::extract::State;
use axum::Json;
use axum::http::HeaderMap;

#[utoipa::path(
    get,
    path = "/api/portfolio/summary",
    responses(
        (status = 200, description = "Portfolio summary", body = PortfolioSummary)
    ),
    tag = "portfolio"
)]
pub async fn get_summary(State(state): State<AppState>, headers: HeaderMap) -> Json<PortfolioSummary> {
    let user_id: i64 = extract_user_id(headers).unwrap();
    let svc = build_portfolio_service(&state, user_id);
    let summary = svc.get_summary().await.unwrap();
    Json(summary)
}

#[utoipa::path(
    get,
    path = "/api/portfolio/holdings",
    responses(
        (status = 200, description = "Portfolio holdings", body = Vec<Portfolio>)
    ),
    tag = "portfolio"
)]
pub async fn get_holdings(State(state): State<AppState>, headers: HeaderMap) -> Json<Vec<Portfolio>> {
    let user_id: i64 = extract_user_id(headers).unwrap();
    let svc = build_portfolio_service(&state, user_id);
    let portfolio = svc.get_portfolio().await.unwrap();
    Json(portfolio)
}

fn extract_user_id(headers: HeaderMap) -> Result<i64, AppError> {
    headers.get(USER_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .ok_or(AppError::MissingUserId)
}

fn build_portfolio_service(state: &AppState, user_id: i64) -> PortfolioService<PostgresStockTradeRepository> {
    let trade_repo = PostgresStockTradeRepository::new(state.db.clone(), user_id);
    PortfolioService::new(trade_repo, Arc::clone(&state.price_service))
}

const USER_ID_HEADER: &str = "X-User-Id";
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("UserId missing in the header")]
    MissingUserId
}