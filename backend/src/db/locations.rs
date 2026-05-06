use sqlx::PgPool;
use uuid::Uuid;

use super::models::{CreateLocationRequest, LocationWithFlag, UpdateLocationRequest};
use crate::errors::{AppError, AppResult};

// ============================================================
// 获取系统默认 + 用户自定义存放地点
// ============================================================

pub async fn list_locations(
    pool: &PgPool,
    user_id: Uuid,
) -> AppResult<Vec<LocationWithFlag>> {
    let locations = sqlx::query_as::<_, LocationWithFlag>(
        r#"SELECT
             id, user_id, name, created_at,
             (user_id IS NULL) AS is_system
           FROM locations
           WHERE user_id IS NULL OR user_id = $1
           ORDER BY is_system DESC, name ASC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(locations)
}

// ============================================================
// 创建自定义存放地点
// ============================================================

pub async fn create_location(
    pool: &PgPool,
    user_id: Uuid,
    req: &CreateLocationRequest,
) -> AppResult<LocationWithFlag> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("地点名称不能为空".to_string()));
    }

    let location = sqlx::query_as::<_, LocationWithFlag>(
        r#"INSERT INTO locations (user_id, name)
           VALUES ($1, $2)
           RETURNING id, user_id, name, created_at, FALSE AS is_system"#,
    )
    .bind(user_id)
    .bind(req.name.trim())
    .fetch_one(pool)
    .await?;

    Ok(location)
}

// ============================================================
// 更新存放地点（只能更新自己的）
// ============================================================

pub async fn update_location(
    pool: &PgPool,
    user_id: Uuid,
    location_id: i32,
    req: &UpdateLocationRequest,
) -> AppResult<LocationWithFlag> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("地点名称不能为空".to_string()));
    }

    // 检查地点是否存在且属于当前用户
    let existing = sqlx::query_as::<_, LocationWithFlag>(
        r#"SELECT id, user_id, name, created_at, (user_id IS NULL) AS is_system
           FROM locations WHERE id = $1"#,
    )
    .bind(location_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("存放地点不存在".to_string()))?;

    if existing.is_system {
        return Err(AppError::Forbidden("不能修改系统默认存放地点".to_string()));
    }

    if existing.user_id != Some(user_id) {
        return Err(AppError::Forbidden("只能修改自己的存放地点".to_string()));
    }

    let location = sqlx::query_as::<_, LocationWithFlag>(
        r#"UPDATE locations SET name = $1
           WHERE id = $2 AND user_id = $3
           RETURNING id, user_id, name, created_at, FALSE AS is_system"#,
    )
    .bind(req.name.trim())
    .bind(location_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(location)
}

// ============================================================
// 删除存放地点（只能删除自己的，不能删除系统默认的）
// ============================================================

pub async fn delete_location(
    pool: &PgPool,
    user_id: Uuid,
    location_id: i32,
) -> AppResult<()> {
    // 检查地点是否存在且属于当前用户
    let existing = sqlx::query_as::<_, LocationWithFlag>(
        r#"SELECT id, user_id, name, created_at, (user_id IS NULL) AS is_system
           FROM locations WHERE id = $1"#,
    )
    .bind(location_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("存放地点不存在".to_string()))?;

    if existing.is_system {
        return Err(AppError::Forbidden("不能删除系统默认存放地点".to_string()));
    }

    if existing.user_id != Some(user_id) {
        return Err(AppError::Forbidden("只能删除自己的存放地点".to_string()));
    }

    // 检查是否有物品使用此地点
    let usage_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM items WHERE location_id = $1",
    )
    .bind(location_id)
    .fetch_one(pool)
    .await?;

    if usage_count.0 > 0 {
        return Err(AppError::BadRequest(format!(
            "该地点下还有 {} 个物品，无法删除",
            usage_count.0
        )));
    }

    sqlx::query("DELETE FROM locations WHERE id = $1 AND user_id = $2")
        .bind(location_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}
