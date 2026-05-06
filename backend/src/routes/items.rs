use axum::extract::{Extension, Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::items;
use crate::db::models::{AuthUser, CreateItemRequest, UpdateItemRequest};
use crate::errors::AppResult;

#[derive(Clone)]
pub struct ItemsState {
    pub pool: PgPool,
}

pub fn items_routes(state: ItemsState) -> Router {
    Router::new()
        .route("/api/items", post(create_item).get(list_items))
        .route("/api/items/expiring", get(list_expiring_items))
        .route(
            "/api/items/:id",
            get(get_item).put(update_item).delete(delete_item),
        )
        .with_state(state)
}

async fn create_item(
    State(state): State<ItemsState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let item = items::create_item(
        &state.pool,
        auth_user.user_id,
        &req.name,
        req.category_id,
        req.location_id,
    )
    .await?;
    Ok(Json(item))
}

async fn list_items(
    State(state): State<ItemsState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<impl IntoResponse> {
    let items = items::list_items(&state.pool, auth_user.user_id).await?;
    Ok(Json(items))
}

async fn get_item(
    State(state): State<ItemsState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let item = items::get_item(&state.pool, auth_user.user_id, id).await?;
    Ok(Json(item))
}

async fn update_item(
    State(state): State<ItemsState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let item = items::update_item(
        &state.pool,
        auth_user.user_id,
        id,
        req.name.as_deref(),
        req.category_id,
        req.location_id,
    )
    .await?;
    Ok(Json(item))
}

async fn delete_item(
    State(state): State<ItemsState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    items::delete_item(&state.pool, auth_user.user_id, id).await?;
    Ok(Json(json!({ "message": "删除成功" })))
}

async fn list_expiring_items(
    State(state): State<ItemsState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<impl IntoResponse> {
    let items = items::get_expiring_items(&state.pool, auth_user.user_id).await?;
    Ok(Json(items))
}
