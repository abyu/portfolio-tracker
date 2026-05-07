use axum::{routing::get, Router};

mod config;
use config::AppConfig;
mod app_state;
use app_state::AppState;
use sqlx::sqlite::SqlitePoolOptions;


#[tokio::main]
async fn main() {
    let config = AppConfig::load().unwrap();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(config.db_url.as_str())
        .await
        .unwrap();

    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    let app = Router::new()
        .route("/", get(index))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{}", config.server_port)
    ).await.unwrap();

    println!("Listening on port {}", config.server_port);
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> String {
    return String::from("Hello")
}