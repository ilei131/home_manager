use sqlx::PgPool;
use uuid::Uuid;

use super::models::{SystemStats, UserStats};
use crate::errors::AppResult;

// ============================================================
// 获取系统统计信息（仅管理员）
// ============================================================

pub async fn get_system_stats(pool: &PgPool) -> AppResult<SystemStats> {
    let stats = sqlx::query_as::<_, SystemStats>(
        r#"SELECT
             (SELECT COUNT(*) FROM users) AS total_users,
             (SELECT COUNT(*) FROM items) AS total_items,
             (SELECT COUNT(*) FROM categories) AS total_categories,
             (SELECT COUNT(*) FROM locations) AS total_locations,
             (SELECT COUNT(*) FROM batches) AS total_batches"#,
    )
    .fetch_one(pool)
    .await?;

    Ok(stats)
}

// ============================================================
// 获取当前用户的统计信息
// ============================================================

pub async fn get_user_stats(pool: &PgPool, user_id: Uuid) -> AppResult<UserStats> {
    let stats = sqlx::query_as::<_, UserStats>(
        r#"SELECT
             (SELECT COUNT(*) FROM items WHERE user_id = $1) AS total_items,
             (SELECT COUNT(*) FROM batches b
              JOIN items i ON i.id = b.item_id
              WHERE i.user_id = $1) AS total_batches,
             (SELECT COALESCE(SUM(b.quantity), 0) FROM batches b
              JOIN items i ON i.id = b.item_id
              WHERE i.user_id = $1) AS total_quantity,
             (SELECT COUNT(*) FROM batches b
              JOIN items i ON i.id = b.item_id
              WHERE i.user_id = $1 AND b.expiry_date < CURRENT_DATE) AS expired_batches"#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(stats)
}
