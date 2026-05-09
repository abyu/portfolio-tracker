use axum::{routing::get, Router};

mod config;
use config::AppConfig;
mod app_state;
use app_state::AppState;
use sqlx::sqlite::SqlitePoolOptions;
mod models;
mod db;
mod routes;
use routes::stock_trade::render_all;
mod service;
mod tasks;
use tasks::price_update_task::PriceUpdateTask;

use crate::service::price_update_service::PriceUpdateService;
use db::{stock_price_repository::StockPriceRepository, stock_trade_repository::StockTradeRepository};
use yahoo_finance_api::YahooConnector;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    let config = AppConfig::load().unwrap();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(config.db_url.as_str())
        .await
        .unwrap();

    let tera = tera::Tera::new("templates/**/*.html").unwrap();

    let yahoo = YahooConnector::new().unwrap();
    let price_service = PriceUpdateService::new(
        StockPriceRepository::new(pool.clone()),
        yahoo
    );

    let task = PriceUpdateTask::new(price_service, StockTradeRepository::new(pool.clone()));

    // wrap in Arc so it can be shared across threads
    let task = std::sync::Arc::new(task);
    let task_clone = Arc::clone(&task);

    // spawn background job
    tokio::spawn(async move {
        // initial run at startup
        if let Err(e) = task_clone.start().await {
            eprintln!("Initial price fetch failed: {}", e);
        }

        // recurring every 20 mins
        let mut interval = time::interval(Duration::from_secs(60 * 20));
        loop {
            interval.tick().await;
            if let Err(e) = task_clone.start().await {
                eprintln!("Price update failed: {}", e);
            }
        }
    });

    let state = AppState {
        db: pool,
        config: config.clone(),
        tera,
    };
    let app = Router::new()
        .route("/", get(render_all))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{}", config.server_port)
    ).await.unwrap();

    println!("Listening on port {}", config.server_port);
    axum::serve(listener, app).await.unwrap();
}
