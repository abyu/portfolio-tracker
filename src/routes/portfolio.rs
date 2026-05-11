use crate::db::postgres_stock_price_repository::PostgresStockPriceRepository;
use crate::db::postgres_stock_trade_repository::PostgresStockTradeRepository;
use crate::models::portfolio::PortfolioSummary;
use crate::app_state::AppState;
use crate::service::portfolio_service::PortfolioService;
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
    let trade_repo = PostgresStockTradeRepository::new(state.db.clone());
    let price_repo = PostgresStockPriceRepository::new(state.db.clone());
    let svc = PortfolioService::new(trade_repo, price_repo);
    let summary = svc.get_summary().await.unwrap();
    Json(summary)
}