use sqlx::PgPool;
use uuid::Uuid;

use super::models::{Batch, CreateBatchRequest, UpdateBatchRequest};
use crate::errors::{AppError, AppResult};

// ============================================================
// 为物品添加批次
// ============================================================

pub async fn create_batch(
    pool: &PgPool,
    item_id: Uuid,
    req: &CreateBatchRequest,
) -> AppResult<Batch> {
    if req.quantity <= 0 {
        return Err(AppError::BadRequest("数量必须大于 0".to_string()));
    }

    let batch = sqlx::query_as::<_, Batch>(
        r#"INSERT INTO batches (item_id, quantity, expiry_date)
           VALUES ($1, $2, $3)
           RETURNING id, item_id, quantity, expiry_date, created_at"#,
    )
    .bind(item_id)
    .bind(req.quantity)
    .bind(req.expiry_date)
    .fetch_one(pool)
    .await?;

    Ok(batch)
}

// ============================================================
// 获取物品的所有批次
// ============================================================

pub async fn list_batches(pool: &PgPool, item_id: Uuid) -> AppResult<Vec<Batch>> {
    let batches = sqlx::query_as::<_, Batch>(
        r#"SELECT id, item_id, quantity, expiry_date, created_at
           FROM batches
           WHERE item_id = $1
           ORDER BY created_at DESC"#,
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;

    Ok(batches)
}

// ============================================================
// 更新批次
// ============================================================

pub async fn update_batch(
    pool: &PgPool,
    batch_id: Uuid,
    req: &UpdateBatchRequest,
) -> AppResult<Batch> {
    // 先获取现有批次信息
    let existing = sqlx::query_as::<_, Batch>(
        r#"SELECT id, item_id, quantity, expiry_date, created_at
           FROM batches WHERE id = $1"#,
    )
    .bind(batch_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("批次不存在".to_string()))?;

    let final_quantity = req.quantity.unwrap_or(existing.quantity);
    let final_expiry_date = req.expiry_date.unwrap_or(existing.expiry_date);

    if final_quantity <= 0 {
        return Err(AppError::BadRequest("数量必须大于 0".to_string()));
    }

    let batch = sqlx::query_as::<_, Batch>(
        r#"UPDATE batches
           SET quantity = $1, expiry_date = $2
           WHERE id = $3
           RETURNING id, item_id, quantity, expiry_date, created_at"#,
    )
    .bind(final_quantity)
    .bind(final_expiry_date)
    .bind(batch_id)
    .fetch_one(pool)
    .await?;

    Ok(batch)
}

// ============================================================
// 删除批次
// ============================================================

pub async fn delete_batch(pool: &PgPool, batch_id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM batches WHERE id = $1")
        .bind(batch_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("批次不存在".to_string()));
    }

    Ok(())
}
