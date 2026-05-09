use crate::app_state::AppState;
use crate::db::{stock_trade_repository::StockTradeRepository, stock_price_repository::StockPriceRepository};
use crate::service::portfolio_service::PortfolioService;
use axum::extract::State;
use axum::response::Html;
use tera::{Value, to_value};
use std::collections::HashMap;

pub async fn render_all(State(state): State<AppState>) -> Html<String> {
    let repo = StockTradeRepository::new(state.db.clone());
    let stock_price_repo = StockPriceRepository::new(state.db.clone());
    let portfolio_service = PortfolioService::new(repo, stock_price_repo);
    let stock_trades = portfolio_service.get_portfolio().await.unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("trades", &stock_trades);
    Html(state.tera.render("index.html", &ctx).unwrap())
}

pub fn cents_to_dollars_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let cents = value.as_i64().unwrap_or(0);
    let dollars = cents as f64 / 100.0;
    Ok(to_value(format!("${:.2}", dollars))?)
}