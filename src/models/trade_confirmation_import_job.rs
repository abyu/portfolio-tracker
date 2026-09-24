#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct TradeConfirmationImportJob {
    pub id: i64,
    pub user_id: i64,
    pub file_name: String,
    pub file_content: Vec<u8>
}

#[derive(Debug, serde::Serialize)]
pub struct NewTradeConfirmationImportJob {
    pub user_id: i64,
    pub file_name: String,
    pub file_content: Vec<u8>
}