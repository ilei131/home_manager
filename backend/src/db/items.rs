use sqlx::PgPool;
use uuid::Uuid;

use super::models::{ItemWithDetails, ItemWithDetailsRow};
use crate::errors::{AppError, AppResult};

// ============================================================
// 创建物品
// ============================================================

pub async fn create_item(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    category_id: i32,
    location_id: i32,
) -> AppResult<ItemWithDetails> {
    let item_row = sqlx::query_as::<_, ItemWithDetailsRow>(
        r#"INSERT INTO items (user_id, name, category_id, location_id)
           VALUES ($1, $2, $3, $4)
           RETURNING
             id, user_id, name, category_id, location_id,
             created_at, updated_at,
             (SELECT c.name FROM categories c WHERE c.id = items.category_id) AS category_name,
             (SELECT l.name FROM locations l WHERE l.id = items.location_id) AS location_name"#,
    )
    .bind(user_id)
    .bind(name)
    .bind(category_id)
    .bind(location_id)
    .fetch_one(pool)
    .await?;

    Ok(item_row.into())
}

// ============================================================
// 获取当前用户的所有物品
// ============================================================

pub async fn list_items(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<ItemWithDetails>> {
    let item_rows = sqlx::query_as::<_, ItemWithDetailsRow>(
        r#"SELECT
             i.id, i.user_id, i.name, i.category_id, i.location_id,
             i.created_at, i.updated_at,
             c.name AS category_name,
             l.name AS location_name
           FROM items i
           JOIN categories c ON c.id = i.category_id
           JOIN locations l ON l.id = i.location_id
           WHERE i.user_id = $1
           ORDER BY i.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut items: Vec<ItemWithDetails> = item_rows.into_iter().map(|row| row.into()).collect();

    // 查询每个物品的批次信息
    for item in &mut items {
        let batches = sqlx::query_as::<_, crate::db::models::Batch>(
            r#"SELECT id, item_id, quantity, expiry_date, created_at FROM batches WHERE item_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(item.id)
        .fetch_all(pool)
        .await?;
        item.batches = batches;
    }

    Ok(items)
}

// ============================================================
// 获取单个物品详情
// ============================================================

pub async fn get_item(
    pool: &PgPool,
    user_id: Uuid,
    item_id: Uuid,
) -> AppResult<ItemWithDetails> {
    let item_row = sqlx::query_as::<_, ItemWithDetailsRow>(
        r#"SELECT
             i.id, i.user_id, i.name, i.category_id, i.location_id,
             i.created_at, i.updated_at,
             c.name AS category_name,
             l.name AS location_name
           FROM items i
           JOIN categories c ON c.id = i.category_id
           JOIN locations l ON l.id = i.location_id
           WHERE i.id = $1 AND i.user_id = $2"#,
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("物品不存在".to_string()))?;

    let mut item: ItemWithDetails = item_row.into();

    // 查询批次信息
    let batches = sqlx::query_as::<_, crate::db::models::Batch>(
        r#"SELECT id, item_id, quantity, expiry_date, created_at FROM batches WHERE item_id = $1 ORDER BY created_at DESC"#,
    )
    .bind(item.id)
    .fetch_all(pool)
    .await?;
    item.batches = batches;

    Ok(item)
}

// ============================================================
// 更新物品
// ============================================================

pub async fn update_item(
    pool: &PgPool,
    user_id: Uuid,
    item_id: Uuid,
    name: Option<&str>,
    category_id: Option<i32>,
    location_id: Option<i32>,
) -> AppResult<ItemWithDetails> {
    let existing = get_item(pool, user_id, item_id).await?;

    let final_name = name.unwrap_or(&existing.name);
    let final_category_id = category_id.unwrap_or(existing.category_id);
    let final_location_id = location_id.unwrap_or(existing.location_id);

    let item_row = sqlx::query_as::<_, ItemWithDetailsRow>(
        r#"UPDATE items
           SET name = $1, category_id = $2, location_id = $3, updated_at = NOW()
           WHERE id = $4 AND user_id = $5
           RETURNING
             id, user_id, name, category_id, location_id,
             created_at, updated_at,
             (SELECT c.name FROM categories c WHERE c.id = items.category_id) AS category_name,
             (SELECT l.name FROM locations l WHERE l.id = items.location_id) AS location_name"#,
    )
    .bind(final_name)
    .bind(final_category_id)
    .bind(final_location_id)
    .bind(item_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(item_row.into())
}

// ============================================================
// 删除物品
// ============================================================

pub async fn delete_item(pool: &PgPool, user_id: Uuid, item_id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM items WHERE id = $1 AND user_id = $2")
        .bind(item_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("物品不存在".to_string()));
    }

    Ok(())
}

// ============================================================
// 获取临期物品（7天内过期）
// ============================================================

pub async fn get_expiring_items(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<ItemWithDetails>> {
    let item_rows = sqlx::query_as::<_, ItemWithDetailsRow>(
        r#"SELECT DISTINCT ON (i.id)
             i.id, i.user_id, i.name, i.category_id, i.location_id,
             i.created_at, i.updated_at,
             c.name AS category_name,
             l.name AS location_name
           FROM items i
           JOIN categories c ON c.id = i.category_id
           JOIN locations l ON l.id = i.location_id
           JOIN batches b ON b.item_id = i.id
           WHERE i.user_id = $1
             AND b.expiry_date IS NOT NULL
             AND b.expiry_date <= NOW() + INTERVAL '7 days'
           ORDER BY i.id, b.expiry_date ASC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut items: Vec<ItemWithDetails> = item_rows.into_iter().map(|row| row.into()).collect();

    for item in &mut items {
        let batches = sqlx::query_as::<_, crate::db::models::Batch>(
            r#"SELECT id, item_id, quantity, expiry_date, created_at FROM batches WHERE item_id = $1 ORDER BY expiry_date ASC"#,
        )
        .bind(item.id)
        .fetch_all(pool)
        .await?;
        item.batches = batches;
    }

    Ok(items)
}
