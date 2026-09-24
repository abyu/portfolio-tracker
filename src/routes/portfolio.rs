use std::sync::Arc;

use crate::app_state::AppState;
use crate::db::postgres_user_stock_trade_repository::PostgresUserStockTradeRepository;
use crate::models::portfolio::{Portfolio, PortfolioSummary};
use crate::routes::headers::extract_user_id;
use crate::service::portfolio_service::{PortfolioService, PortfolioServiceTrait};
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;

#[utoipa::path(
    get,
    path = "/api/portfolio/summary",
    params(
        ("X-User-Id" = i64, Header, description = "User ID")
    ),
    responses(
        (status = 200, description = "Portfolio summary", body = PortfolioSummary)
    ),
    tag = "portfolio"
)]
pub async fn get_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<PortfolioSummary> {
    let user_id: i64 = extract_user_id(headers).unwrap();
    let svc = build_portfolio_service(&state, user_id);
    let summary = svc.get_summary().await.unwrap();
    Json(summary)
}

#[utoipa::path(
    get,
    path = "/api/portfolio/holdings",
    params(
        ("X-User-Id" = i64, Header, description = "User ID")
    ),
    responses(
        (status = 200, description = "Portfolio holdings", body = Vec<Portfolio>)
    ),
    tag = "portfolio"
)]
pub async fn get_holdings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<Vec<Portfolio>> {
    let user_id: i64 = extract_user_id(headers).unwrap();
    let svc = build_portfolio_service(&state, user_id);
    let portfolio = svc.get_portfolio().await.unwrap();
    Json(portfolio)
}

fn build_portfolio_service(
    state: &AppState,
    user_id: i64,
) -> PortfolioService<PostgresUserStockTradeRepository> {
    let trade_repo = PostgresUserStockTradeRepository::new(state.db.clone(), user_id);
    PortfolioService::new(trade_repo, Arc::clone(&state.price_service))
}
