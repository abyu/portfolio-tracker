use axum::extract::Multipart;
use axum::{extract::State, http::HeaderMap};
use axum::Json;
use chrono::naive::serde::ts_seconds;
use utoipa::ToSchema;
use crate::routes::headers::extract_user_id;
use crate::service::portfolio_service::UserStockTradeRepository;
use crate::db::postgres_user_stock_trade_repository::PostgresUserStockTradeRepository;
use crate::app_state::AppState;
use crate::import::csv_parser;
use crate::import::trade_confirmation_parser;

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


#[utoipa::path(
    post,
    path = "/api/trades/import_trade_confirmation",
    request_body(
        content_type = "multipart/form-data",
        content = inline(ImportRequest)
    ),
    params(
        ("X-User-Id" = i64, Header, description = "User ID")
    ),
    responses(
        (status = 201, description = "Import trade a from a trade confirmation", body = ImportResult)
    ),
    tag = "importtradeconfirmation"
)]
pub async fn import_trade_confirmation(State(state): State<AppState>,  headers: HeaderMap, mut multipart: Multipart) -> Json<ImportResult> {
    let user_id: i64 = extract_user_id(headers).unwrap();
    let pdf_content = multipart.next_field().await.unwrap().unwrap().bytes().await.unwrap();
    let text = trade_confirmation_parser::parse_trade_confirmation(&pdf_content[..]);
    Json(ImportResult { imported_rows: text.iter().len()})
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