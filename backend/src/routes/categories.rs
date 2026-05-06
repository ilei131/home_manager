use axum::extract::{Extension, Path, State};
use axum::response::IntoResponse;
use axum::routing::{post, put};
use axum::{Json, Router};
use serde_json::json;
use sqlx::PgPool;

use crate::db::categories;
use crate::db::models::{AuthUser, CreateCategoryRequest, UpdateCategoryRequest};
use crate::errors::AppResult;

#[derive(Clone)]
pub struct CategoriesState {
    pub pool: PgPool,
}

pub fn categories_routes(state: CategoriesState) -> Router {
    Router::new()
        .route(
            "/api/categories",
            post(create_category).get(list_categories),
        )
        .route(
            "/api/categories/:id",
            put(update_category).delete(delete_category),
        )
        .with_state(state)
}

async fn create_category(
    State(state): State<CategoriesState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateCategoryRequest>,
) -> AppResult<impl IntoResponse> {
    let category = categories::create_category(&state.pool, auth_user.user_id, &req).await?;
    Ok(Json(category))
}

async fn list_categories(
    State(state): State<CategoriesState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<impl IntoResponse> {
    let categories_list = categories::list_categories(&state.pool, auth_user.user_id).await?;
    Ok(Json(categories_list))
}

async fn update_category(
    State(state): State<CategoriesState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCategoryRequest>,
) -> AppResult<impl IntoResponse> {
    let category = categories::update_category(&state.pool, auth_user.user_id, id, &req).await?;
    Ok(Json(category))
}

async fn delete_category(
    State(state): State<CategoriesState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> AppResult<impl IntoResponse> {
    categories::delete_category(&state.pool, auth_user.user_id, id).await?;
    Ok(Json(json!({ "message": "删除成功" })))
}
