use crate::models::portfolio::{Portfolio, PortfolioSummary};
use crate::app_state::AppState;
use axum::extract::State;
use axum::Json;

#[utoipa::path(
    get,
    path = "/api/portfolio/summary",
    responses(
        (status = 200, description = "Portfolio summary", body = PortfolioSummary)
    ),
    tag = "portfolio"
)]
pub async fn get_summary(State(state): State<AppState>) -> Json<PortfolioSummary> {
    let svc = state.portfolio_service.clone();
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
pub async fn get_holdings(State(state): State<AppState>) -> Json<Vec<Portfolio>> {
    let svc = state.portfolio_service.clone();
    let portfolio = svc.get_portfolio().await.unwrap();
    Json(portfolio)
}