use axum::extract::{Extension, State};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::get;
use sqlx::PgPool;

use crate::db::models::AuthUser;
use crate::db::stats;
use crate::errors::AppResult;
use crate::middleware::require_admin;

#[derive(Clone)]
pub struct StatsState {
    pub pool: PgPool,
}

pub fn stats_routes(state: StatsState) -> Router {
    Router::new()
        .route("/api/stats/system", get(get_system_stats))
        .route("/api/stats/user", get(get_user_stats))
        .with_state(state)
}

async fn get_system_stats(
    State(state): State<StatsState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<impl IntoResponse> {
    require_admin(&auth_user)?;
    let stats = stats::get_system_stats(&state.pool).await?;
    Ok(Json(stats))
}

async fn get_user_stats(
    State(state): State<StatsState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<impl IntoResponse> {
    let stats = stats::get_user_stats(&state.pool, auth_user.user_id).await?;
    Ok(Json(stats))
}
