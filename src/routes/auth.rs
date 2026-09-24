use axum::{Json, extract::State};

use crate::app_state::AppState;

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body(
        content_type = "application/json",
        content = LoginRequest
    ),
    responses(
        (status = 200, description = "Session token", body = LoginResponse)
    ),
    tag = "portfolio"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Json<LoginResponse> {
    let user = state
        .user_service
        .validate_credentials(request.email, request.password)
        .await
        .unwrap();
    let token = state.jwt_service.generate_token(user.id).unwrap();
    Json(LoginResponse { token })
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub token: String,
}
