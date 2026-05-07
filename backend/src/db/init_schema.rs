use sqlx::PgPool;

use crate::errors::AppResult;

pub async fn init_schema(pool: &PgPool) -> AppResult<()> {
    tracing::info!("正在初始化数据库表结构...");

    // 用户表
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(50) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            role TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('admin', 'user')),
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await?;

    // 分类表
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS categories (
            id SERIAL PRIMARY KEY,
            user_id UUID REFERENCES users(id) ON DELETE CASCADE,
            name VARCHAR(100) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            UNIQUE(user_id, name)
        )"#,
    )
    .execute(pool)
    .await?;

    // 存放地点表
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS locations (
            id SERIAL PRIMARY KEY,
            user_id UUID REFERENCES users(id) ON DELETE CASCADE,
            name VARCHAR(100) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            UNIQUE(user_id, name)
        )"#,
    )
    .execute(pool)
    .await?;

    // 物品表
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS items (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            name VARCHAR(200) NOT NULL,
            category_id INTEGER NOT NULL REFERENCES categories(id),
            location_id INTEGER NOT NULL REFERENCES locations(id),
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await?;

    // 批次表
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS batches (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            quantity INTEGER NOT NULL DEFAULT 1,
            expiry_date DATE,
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await?;

    // 索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_user_id ON items(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_batches_item_id ON batches(item_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_categories_user_id ON categories(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_locations_user_id ON locations(user_id)")
        .execute(pool)
        .await?;

    tracing::info!("数据库表结构初始化完成");
    Ok(())
}
