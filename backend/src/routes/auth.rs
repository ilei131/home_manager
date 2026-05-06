use axum::extract::{Extension, State};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::Deserialize;
use sqlx::PgPool;

use crate::db::auth::{self, get_user_by_id};
use crate::db::models::{AuthUser, RegisterRequest, UserResponse};
use crate::errors::AppResult;

#[derive(Clone)]
pub struct AuthState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub fn auth_routes(state: AuthState) -> Router {
    Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(me))
        .with_state(state)
}

async fn register(
    State(state): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = auth::register(&state.pool, req, &state.jwt_secret).await?;
    Ok(Json(resp))
}

async fn login(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = auth::login(&state.pool, &req.username, &req.password, &state.jwt_secret).await?;
    Ok(Json(resp))
}

async fn me(
    State(state): State<AuthState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<impl IntoResponse> {
    let user = get_user_by_id(&state.pool, auth_user.user_id).await?;
    let resp = UserResponse::from(user);
    Ok(Json(resp))
}
