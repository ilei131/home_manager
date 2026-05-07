use sqlx::PgPool;

use crate::errors::AppResult;

const DEFAULT_CATEGORIES: &[&str] = &[
    "食品", "饮料", "日用品", "调味品", "清洁用品",
    "个人护理", "文具", "工具", "药品", "其他",
];

const DEFAULT_LOCATIONS: &[&str] = &[
    "厨房", "冰箱", "卫生间", "卧室", "客厅",
    "阳台", "书房", "储物间", "车库", "其他",
];

pub async fn init_default_categories(pool: &PgPool) -> AppResult<()> {
    for name in DEFAULT_CATEGORIES {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE user_id IS NULL AND name = $1)",
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        if !exists {
            sqlx::query("INSERT INTO categories (name) VALUES ($1)")
                .bind(name)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

pub async fn init_default_locations(pool: &PgPool) -> AppResult<()> {
    for name in DEFAULT_LOCATIONS {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM locations WHERE user_id IS NULL AND name = $1)",
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        if !exists {
            sqlx::query("INSERT INTO locations (name) VALUES ($1)")
                .bind(name)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

pub async fn init_default_data(pool: &PgPool) -> AppResult<()> {
    tracing::info!("正在初始化默认数据...");
    
    init_default_categories(pool).await?;
    init_default_locations(pool).await?;
    
    tracing::info!("默认数据初始化完成");
    Ok(())
}
