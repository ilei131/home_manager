use axum::extract::{Extension, State};
use axum::middleware::from_fn_with_state;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use sqlx::PgPool;

use crate::db::auth::{self, get_user_by_id};
use crate::db::models::{AuthUser, RegisterRequest, UserResponse};
use crate::errors::AppResult;
use crate::middleware::{require_admin, RateLimitState};

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

pub fn auth_routes(state: AuthState, rate_limit_state: RateLimitState) -> Router {
    Router::new()
        .route(
            "/api/auth/register",
            post(register).route_layer(from_fn_with_state(
                rate_limit_state.clone(),
                crate::middleware::register_rate_limit,
            )),
        )
        .route(
            "/api/auth/login",
            post(login).route_layer(from_fn_with_state(
                rate_limit_state,
                crate::middleware::login_rate_limit,
            )),
        )
        .with_state(state)
}

pub fn me_route(state: AuthState) -> Router {
    Router::new()
        .route("/api/auth/me", get(me))
        .route("/api/auth/users", get(get_users))
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

async fn get_users(
    State(state): State<AuthState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<impl IntoResponse> {
    require_admin(&auth_user)?;
    let users = auth::get_all_users(&state.pool).await?;
    Ok(Json(users))
}
