use axum::extract::{Extension, Path, State};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{post, put};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::batches;
use crate::db::models::{AuthUser, Batch, CreateBatchRequest, UpdateBatchRequest};
use crate::db::items;
use crate::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct BatchesState {
    pub pool: PgPool,
}

pub fn batches_routes(state: BatchesState) -> Router {
    Router::new()
        .route(
            "/api/items/:item_id/batches",
            post(create_batch).get(list_batches),
        )
        .route(
            "/api/batches/:id",
            put(update_batch).delete(delete_batch),
        )
        .with_state(state)
}

async fn create_batch(
    State(state): State<BatchesState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    Json(req): Json<CreateBatchRequest>,
) -> AppResult<impl IntoResponse> {
    let item = items::get_item(&state.pool, auth_user.user_id, item_id).await?;

    let batch = batches::create_batch(&state.pool, item.id, &req).await?;
    Ok(Json(batch))
}

async fn list_batches(
    State(state): State<BatchesState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    items::get_item(&state.pool, auth_user.user_id, item_id).await?;

    let batches_list = batches::list_batches(&state.pool, item_id).await?;
    Ok(Json(batches_list))
}

async fn update_batch(
    State(state): State<BatchesState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBatchRequest>,
) -> AppResult<impl IntoResponse> {
    let batch_detail = sqlx::query_as::<_, Batch>(
        r#"SELECT id, item_id, quantity, expiry_date, created_at
           FROM batches WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("批次不存在".to_string()))?;

    items::get_item(&state.pool, auth_user.user_id, batch_detail.item_id).await?;

    let updated = batches::update_batch(&state.pool, id, &req).await?;
    Ok(Json(updated))
}

async fn delete_batch(
    State(state): State<BatchesState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let batch_detail = sqlx::query_as::<_, Batch>(
        r#"SELECT id, item_id, quantity, expiry_date, created_at
           FROM batches WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("批次不存在".to_string()))?;

    items::get_item(&state.pool, auth_user.user_id, batch_detail.item_id).await?;

    batches::delete_batch(&state.pool, id).await?;
    Ok(Json(json!({ "message": "删除成功" })))
}
