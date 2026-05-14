use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::portfolio::get_summary,
        crate::routes::portfolio::get_holdings,
        crate::routes::import::import_csv,
    ),
    components(
        schemas(
            crate::models::portfolio::PortfolioSummary,
            crate::models::portfolio::Portfolio,
            crate::routes::import::ImportResult
        )
    ),
    tags(
        (name = "portfolio", description = "Portfolio management")
    )
)]
pub struct ApiDoc;