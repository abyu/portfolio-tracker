use axum::http::HeaderMap;

const USER_ID_HEADER: &str = "X-User-Id";
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("UserId missing in the header")]
    MissingUserId
}

pub fn extract_user_id(headers: HeaderMap) -> Result<i64, AppError> {
    headers.get(USER_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .ok_or(AppError::MissingUserId)
}