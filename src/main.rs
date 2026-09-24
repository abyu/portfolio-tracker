use axum::routing::post;
use axum::{Router, routing::get};

mod config;
use config::AppConfig;
mod app_state;
use app_state::AppState;
use sqlx::postgres::PgPoolOptions;
mod db;
mod models;
mod routes;
use routes::auth::login;
use routes::import::{import_csv, import_trade_confirmation};
use routes::portfolio::{get_holdings, get_summary};
mod service;
mod tasks;
use tasks::price_update_task::PriceUpdateTask;

use crate::clients::ollama::{Ollama, OllamaHttpClient};
use crate::db::postgres_stock_trade_repository::PostgresStockTradeRepository;
use crate::db::postgres_user_repository::PostgresUserRepository;
use crate::service::jwt_service::JwtService;
use crate::service::price_service::PriceService;
use crate::service::ticker_price_service::TickerPriceService;
use crate::service::user_service::UserService;
use db::postgres_stock_price_repository::PostgresStockPriceRepository;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use yahoo_finance_api::YahooConnector;
mod api_doc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod clients;
mod import;

#[tokio::main]
async fn main() {
    let config = AppConfig::load().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(config.db_url.as_str())
        .await
        .unwrap();

    // run migrations on startup
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let yahoo = YahooConnector::new().unwrap();
    let price_service = Arc::new(TickerPriceService::new(
        PostgresStockPriceRepository::new(pool.clone()),
        yahoo,
    ));
    let stock_repo = PostgresStockTradeRepository::new(pool.clone());

    let task = PriceUpdateTask::new(
        Arc::clone(&price_service) as Arc<dyn PriceService>,
        stock_repo,
    );

    // wrap in Arc so it can be shared across threads
    let task = std::sync::Arc::new(task);
    let task_clone = Arc::clone(&task);

    let user_repo = PostgresUserRepository::new(pool.clone());
    let user_service = Arc::new(UserService::new(user_repo));
    let jwt_service = Arc::new(JwtService::new(config.jwt_secret));

    let ollama_client = Arc::new(Ollama::new(
        reqwest::Client::new(),
        config.ollama_url,
        config.ollama_model,
    ));

    // spawn background jobs
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
        price_service: price_service,
        jwt_service: jwt_service,
        user_service: user_service,
        ollama_client: ollama_client,
    };
    let app = Router::new()
        .route("/api/portfolio/summary", get(get_summary))
        .route("/api/portfolio/holdings", get(get_holdings))
        .route("/api/trades/import", post(import_csv))
        .route(
            "/api/trades/import_trade_confirmation",
            post(import_trade_confirmation),
        )
        .route("/api/auth/login", post(login))
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_doc::ApiDoc::openapi()),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server_port))
        .await
        .unwrap();

    println!("Listening on port {}", config.server_port);
    axum::serve(listener, app).await.unwrap();
}
