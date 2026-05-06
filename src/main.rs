use axum::{routing::get, Router};

mod config;
use config::AppConfig;

#[tokio::main]
async fn main() {
    let config = AppConfig::load().unwrap();

    let app = Router::new()
        .route("/", get(index));

    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{}", config.server_port)
    ).await.unwrap();

    println!("Listening on port {}", config.server_port);
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> String {
    return String::from("Hello")
}