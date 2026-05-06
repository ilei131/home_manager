use sqlx::PgPool;
use uuid::Uuid;

use super::models::{CategoryWithFlag, CreateCategoryRequest, UpdateCategoryRequest};
use crate::errors::{AppError, AppResult};

// ============================================================
// 获取系统默认 + 用户自定义分类
// ============================================================

pub async fn list_categories(
    pool: &PgPool,
    user_id: Uuid,
) -> AppResult<Vec<CategoryWithFlag>> {
    let categories = sqlx::query_as::<_, CategoryWithFlag>(
        r#"SELECT
             id, user_id, name, created_at,
             (user_id IS NULL) AS is_system
           FROM categories
           WHERE user_id IS NULL OR user_id = $1
           ORDER BY is_system DESC, name ASC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

// ============================================================
// 创建自定义分类
// ============================================================

pub async fn create_category(
    pool: &PgPool,
    user_id: Uuid,
    req: &CreateCategoryRequest,
) -> AppResult<CategoryWithFlag> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("分类名称不能为空".to_string()));
    }

    let category = sqlx::query_as::<_, CategoryWithFlag>(
        r#"INSERT INTO categories (user_id, name)
           VALUES ($1, $2)
           RETURNING id, user_id, name, created_at, FALSE AS is_system"#,
    )
    .bind(user_id)
    .bind(req.name.trim())
    .fetch_one(pool)
    .await?;

    Ok(category)
}

// ============================================================
// 更新分类（只能更新自己的）
// ============================================================

pub async fn update_category(
    pool: &PgPool,
    user_id: Uuid,
    category_id: i32,
    req: &UpdateCategoryRequest,
) -> AppResult<CategoryWithFlag> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("分类名称不能为空".to_string()));
    }

    // 检查分类是否存在且属于当前用户
    let existing = sqlx::query_as::<_, CategoryWithFlag>(
        r#"SELECT id, user_id, name, created_at, (user_id IS NULL) AS is_system
           FROM categories WHERE id = $1"#,
    )
    .bind(category_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("分类不存在".to_string()))?;

    if existing.is_system {
        return Err(AppError::Forbidden("不能修改系统默认分类".to_string()));
    }

    if existing.user_id != Some(user_id) {
        return Err(AppError::Forbidden("只能修改自己的分类".to_string()));
    }

    let category = sqlx::query_as::<_, CategoryWithFlag>(
        r#"UPDATE categories SET name = $1
           WHERE id = $2 AND user_id = $3
           RETURNING id, user_id, name, created_at, FALSE AS is_system"#,
    )
    .bind(req.name.trim())
    .bind(category_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(category)
}

// ============================================================
// 删除分类（只能删除自己的，不能删除系统默认的）
// ============================================================

pub async fn delete_category(
    pool: &PgPool,
    user_id: Uuid,
    category_id: i32,
) -> AppResult<()> {
    // 检查分类是否存在且属于当前用户
    let existing = sqlx::query_as::<_, CategoryWithFlag>(
        r#"SELECT id, user_id, name, created_at, (user_id IS NULL) AS is_system
           FROM categories WHERE id = $1"#,
    )
    .bind(category_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("分类不存在".to_string()))?;

    if existing.is_system {
        return Err(AppError::Forbidden("不能删除系统默认分类".to_string()));
    }

    if existing.user_id != Some(user_id) {
        return Err(AppError::Forbidden("只能删除自己的分类".to_string()));
    }

    // 检查是否有物品使用此分类
    let usage_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM items WHERE category_id = $1",
    )
    .bind(category_id)
    .fetch_one(pool)
    .await?;

    if usage_count.0 > 0 {
        return Err(AppError::BadRequest(format!(
            "该分类下还有 {} 个物品，无法删除",
            usage_count.0
        )));
    }

    sqlx::query("DELETE FROM categories WHERE id = $1 AND user_id = $2")
        .bind(category_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}
