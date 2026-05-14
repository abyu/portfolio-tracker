use axum::extract::Multipart;
use axum::{extract::State, http::HeaderMap};
use axum::Json;
use utoipa::ToSchema;
use crate::routes::headers::extract_user_id;
use crate::service::portfolio_service::UserStockTradeRepository;
use crate::db::postgres_user_stock_trade_repository::PostgresUserStockTradeRepository;
use crate::app_state::AppState;
use crate::import::csv_parser;

#[utoipa::path(
    post,
    path = "/api/trades/import",
    request_body(
        content_type = "multipart/form-data",
        content = inline(ImportRequest)
    ),
    params(
        ("X-User-Id" = i64, Header, description = "User ID")
    ),
    responses(
        (status = 201, description = "Import trades from csv", body = ImportResult)
    ),
    tag = "importcsv"
)]
pub async fn import_csv(State(state): State<AppState>, headers: HeaderMap, mut multipart: Multipart) -> Json<ImportResult> {
    let user_id: i64 = extract_user_id(headers).unwrap();
    let csv_content = multipart.next_field().await.unwrap().unwrap().text().await.unwrap();
    let trades = csv_parser::parse_csv_to_trades(&csv_content).unwrap();
    let repo = PostgresUserStockTradeRepository::new(state.db.clone(), user_id);
    let count = trades.len();
    for trade in trades {
        repo.save_trade(trade).await.unwrap();
    }    
    Json(ImportResult { imported_rows: count })
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct ImportResult {
    imported_rows: usize
}

#[derive(utoipa::ToSchema)]
pub struct ImportRequest {
    #[schema(format = Binary)]
    pub file: Vec<u8>,
}