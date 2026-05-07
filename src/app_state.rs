use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub config: AppConfig,
    pub tera: tera::Tera,
}