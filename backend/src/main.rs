mod config;
mod db;
mod errors;
mod middleware;
mod routes;

use axum::middleware::from_fn_with_state;
use axum::routing::get;
use dotenvy;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::str::FromStr;
use tower_http::cors::{Any, CorsLayer};

use crate::middleware::JwtSecret;
use config::Config;
use routes::auth::AuthState;
use routes::batches::BatchesState;
use routes::categories::CategoriesState;
use routes::items::ItemsState;
use routes::locations::LocationsState;
use routes::stats::StatsState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载 .env 文件
    dotenvy::dotenv().ok();

    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "home_manager_backend=debug,tower_http=debug,axum=trace"
                    .parse()
                    .unwrap()
            }),
        )
        .init();

    // 加载配置
    let config = Config::from_env();

    // 自动创建数据库（如果不存在）
    create_database_if_not_exists(&config.database_url).await?;

    tracing::info!("正在连接数据库...");

    // 创建数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("数据库连接成功");

    // 运行数据库迁移
    tracing::info!("正在运行数据库迁移...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("数据库迁移完成");

    // 确保默认管理员用户存在
    db::auth::ensure_admin(&pool, &config.admin_default_password).await?;

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建应用路由
    let jwt_secret = JwtSecret(config.jwt_secret.clone());

    // 公开路由（不需要认证）
    let public_routes = routes::auth::auth_routes(AuthState {
        pool: pool.clone(),
        jwt_secret: config.jwt_secret.clone(),
    });

    // 受保护路由（需要认证）
    let protected_routes = axum::Router::new()
        // 健康检查
        .route("/api/health", get(|| async { "OK" }))
        // 物品路由
        .merge(routes::items::items_routes(ItemsState {
            pool: pool.clone(),
        }))
        // 分类路由
        .merge(routes::categories::categories_routes(CategoriesState {
            pool: pool.clone(),
        }))
        // 地点路由
        .merge(routes::locations::locations_routes(LocationsState {
            pool: pool.clone(),
        }))
        // 批次路由
        .merge(routes::batches::batches_routes(BatchesState {
            pool: pool.clone(),
        }))
        // 统计路由
        .merge(routes::stats::stats_routes(StatsState {
            pool: pool.clone(),
        }))
        // 应用认证中间件
        .route_layer(from_fn_with_state(
            jwt_secret,
            crate::middleware::auth_middleware,
        ));

    let app = axum::Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors);

    // 启动服务器
    let addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!("服务器启动于 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================
// 自动创建数据库（如果不存在）
// ============================================================

async fn create_database_if_not_exists(database_url: &str) -> anyhow::Result<()> {
    tracing::info!("检查数据库是否存在...");

    let options = PgConnectOptions::from_str(database_url)?;

    let db_name = options
        .get_database()
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL 中未指定数据库名称"))?
        .to_string();

    let options_without_db = options.database("postgres");

    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options_without_db)
        .await?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)",
    )
    .bind(&db_name)
    .fetch_one(&admin_pool)
    .await?;

    if !exists {
        tracing::info!("数据库 '{}' 不存在，正在创建...", db_name);
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await?;
        tracing::info!("数据库 '{}' 创建成功", db_name);
    } else {
        tracing::info!("数据库 '{}' 已存在", db_name);
    }

    admin_pool.close().await;

    Ok(())
}
