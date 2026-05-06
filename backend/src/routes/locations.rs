use axum::extract::{Extension, Path, State};
use axum::response::IntoResponse;
use axum::routing::{post, put};
use axum::{Json, Router};
use serde_json::json;
use sqlx::PgPool;

use crate::db::locations;
use crate::db::models::{AuthUser, CreateLocationRequest, UpdateLocationRequest};
use crate::errors::AppResult;

#[derive(Clone)]
pub struct LocationsState {
    pub pool: PgPool,
}

pub fn locations_routes(state: LocationsState) -> Router {
    Router::new()
        .route("/api/locations", post(create_location).get(list_locations))
        .route(
            "/api/locations/:id",
            put(update_location).delete(delete_location),
        )
        .with_state(state)
}

async fn create_location(
    State(state): State<LocationsState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateLocationRequest>,
) -> AppResult<impl IntoResponse> {
    let location = locations::create_location(&state.pool, auth_user.user_id, &req).await?;
    Ok(Json(location))
}

async fn list_locations(
    State(state): State<LocationsState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<impl IntoResponse> {
    let locations_list = locations::list_locations(&state.pool, auth_user.user_id).await?;
    Ok(Json(locations_list))
}

async fn update_location(
    State(state): State<LocationsState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateLocationRequest>,
) -> AppResult<impl IntoResponse> {
    let location = locations::update_location(&state.pool, auth_user.user_id, id, &req).await?;
    Ok(Json(location))
}

async fn delete_location(
    State(state): State<LocationsState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> AppResult<impl IntoResponse> {
    locations::delete_location(&state.pool, auth_user.user_id, id).await?;
    Ok(Json(json!({ "message": "删除成功" })))
}
