#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub auth: Option<UserAuth>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct UserAuth {
    pub id: i64,
    pub user_id: i64,
    pub provider: String,
    pub provider_id: String,
    pub credential: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
