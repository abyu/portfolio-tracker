use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::portfolio::get_summary,
    ),
    components(
        schemas(
            crate::models::portfolio::PortfolioSummary,
            crate::models::portfolio::Portfolio,
        )
    ),
    tags(
        (name = "portfolio", description = "Portfolio management")
    )
)]
pub struct ApiDoc;