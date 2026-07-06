use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::portfolio::get_summary,
        crate::routes::portfolio::get_holdings,
        crate::routes::import::import_csv,
        crate::routes::import::import_trade_confirmation,
        crate::routes::auth::login,
    ),
    components(
        schemas(
            crate::models::portfolio::PortfolioSummary,
            crate::models::portfolio::Portfolio,
            crate::routes::import::ImportResult,
            crate::routes::auth::LoginRequest,
            crate::routes::auth::LoginResponse,
        )
    ),
    tags(
        (name = "portfolio", description = "Portfolio management")
    )
)]
pub struct ApiDoc;