use crate::app_state::AppState;
use crate::db::stock_trade_repository::StockTradeRepository;
use axum::extract::State;
use axum::response::Html;

pub async fn render_all(state: State<AppState>) -> Html<String> {
    let repo = StockTradeRepository::new(state.db.clone());
    let stock_trades = repo.get_all_trades().await.unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("trades", &stock_trades);
    Html(state.tera.render("index.html", &ctx).unwrap())

}