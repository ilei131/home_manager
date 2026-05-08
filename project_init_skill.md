
# project_init 技能

## 技能描述

初始化一个前后端分离的全栈项目模板，包含 Rust 后端（PostgreSQL）和 React 前端，支持限流机制、统一错误码、Docker 部署等特性。

## 参数配置

| 参数名 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| project_name | string | - | 必填，项目名称，将用作目录名和包名 |
| author | string | Developer | 作者名称 |
| server_port | string | 3000 | 后端服务端口 |
| database_name | string | app_db | 数据库名称 |

## 技术栈

### 后端
- **语言**: Rust
- **框架**: Axum
- **数据库**: PostgreSQL
- **认证**: JWT
- **限流**: Governor

### 前端
- **语言**: TypeScript
- **框架**: React
- **构建工具**: Vite
- **路由**: React Router
- **UI**: Lucide React Icons

## 功能特性

- ✅ 前后端分离
- ✅ 用户认证（登录/注册）
- ✅ JWT 令牌验证
- ✅ 接口限流机制（登录5次/分钟、注册3次/分钟、通用30次/分钟）
- ✅ 统一错误码处理
- ✅ 环境变量配置
- ✅ Docker 部署支持

## 项目结构

```
{{project_name}}/
├── backend/              # Rust 后端
│   ├── src/
│   │   ├── db/          # 数据库操作
│   │   │   ├── auth.rs
│   │   │   ├── init_data.rs
│   │   │   └── init_schema.rs
│   │   ├── routes/      # API 路由
│   │   │   ├── auth.rs
│   │   │   └── mod.rs
│   │   ├── middleware/  # 中间件
│   │   │   └── mod.rs
│   │   ├── config.rs    # 配置管理
│   │   ├── errors.rs    # 错误处理
│   │   └── main.rs      # 入口文件
│   ├── .env.example
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/             # React 前端
│   ├── src/
│   │   ├── api/         # API 调用
│   │   ├── components/  # 组件
│   │   ├── pages/       # 页面
│   │   ├── hooks/       # 自定义 Hooks
│   │   ├── errors/      # 错误码映射
│   │   ├── styles/      # 样式文件
│   │   └── types/       # 类型定义
│   ├── public/          # 静态资源
│   ├── index.html
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── Dockerfile
├── docker-compose.yml
├── docker-compose.dev.yml
└── README.md
```

## 生成的文件内容

### 后端文件

**Cargo.toml**
```toml
[package]
name = "{{project_name}}-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tower-http = { version = "0.6", features = ["cors"] }
anyhow = "1"
thiserror = "1"
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
bcrypt = "0.17"
governor = "0.6"
tower-governor = "0.4"
```

**src/main.rs**
```rust
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "{{project_name}}=debug,tower_http=debug,axum=trace"
                    .parse()
                    .unwrap()
            }),
        )
        .init();

    let config = Config::from_env();
    let is_new_db = create_database_if_not_exists(&config.database_url).await?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    if is_new_db {
        db::init_schema::init_schema(&pool).await?;
        db::init_data::init_default_data(&pool).await?;
        db::auth::ensure_admin(&pool, &config.admin_default_password).await?;
    } else {
        db::auth::ensure_admin(&pool, &config.admin_default_password).await?;
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let jwt_secret = JwtSecret(config.jwt_secret.clone());
    let rate_limit_state = crate::middleware::RateLimitState::new();

    let public_routes = routes::auth::auth_routes(
        AuthState {
            pool: pool.clone(),
            jwt_secret: config.jwt_secret.clone(),
        },
        rate_limit_state.clone(),
    );

    let protected_routes = axum::Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .merge(routes::auth::me_route(AuthState {
            pool: pool.clone(),
            jwt_secret: config.jwt_secret.clone(),
        }))
        .route_layer(from_fn_with_state(
            jwt_secret,
            crate::middleware::auth_middleware,
        ))
        .route_layer(from_fn_with_state(
            rate_limit_state,
            crate::middleware::general_rate_limit,
        ));

    let app = axum::Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!("服务器启动于 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_database_if_not_exists(database_url: &str) -> anyhow::Result<bool> {
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

    let is_new = !exists;
    if is_new {
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await?;
    }

    admin_pool.close().await;
    Ok(is_new)
}
```

**src/config.rs**
```rust
use dotenvy::var;

#[derive(Clone)]
pub struct Config {
    pub server_port: u16,
    pub jwt_secret: String,
    pub database_url: String,
    pub admin_default_password: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            server_port: var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string()).parse().unwrap(),
            jwt_secret: var("JWT_SECRET").expect("JWT_SECRET 必须设置"),
            database_url: var("DATABASE_URL").expect("DATABASE_URL 必须设置"),
            admin_default_password: var("ADMIN_DEFAULT_PASSWORD").unwrap_or_else(|_| "admin123".to_string()),
        }
    }
}
```

**src/errors.rs**
```rust
use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("认证失败")]
    AuthError,
    #[error("权限不足")]
    Forbidden,
    #[error("请求过于频繁")]
    RateLimited,
    #[error("数据已存在")]
    AlreadyExists,
    #[error("数据不存在")]
    NotFound,
    #[error("请求参数错误")]
    BadRequest,
    #[error("数据库错误")]
    DatabaseError(#[from] sqlx::Error),
    #[error("未知错误")]
    InternalError,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match self {
            AppError::AuthError => (StatusCode::UNAUTHORIZED, "ERR_UNAUTHORIZED", "认证失败"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "ERR_FORBIDDEN", "权限不足"),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "ERR_TOO_MANY_REQUESTS", "请求过于频繁，请稍后再试"),
            AppError::AlreadyExists => (StatusCode::CONFLICT, "ERR_ALREADY_EXISTS", "数据已存在"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "ERR_NOT_FOUND", "数据不存在"),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, "ERR_BAD_REQUEST", "请求参数错误"),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "ERR_DATABASE_ERROR", "数据库错误"),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "ERR_INTERNAL_ERROR", "服务器内部错误"),
        };

        let body = serde_json::to_string(&ErrorResponse { code: code.to_string(), message: message.to_string() }).unwrap();
        (status, [("Content-Type", "application/json")], body)
    }
}
```

**src/middleware.rs**
```rust
use axum::{http::Request, middleware::Next, response::Response};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type LimiterStore = Arc<RwLock<HashMap<String, RateLimiter<DefaultClock>>>>;

#[derive(Clone)]
pub struct RateLimitState {
    login_limiter: Arc<RateLimiter<DefaultClock>>,
    register_limiter: Arc<RateLimiter<DefaultClock>>,
    general_limiters: LimiterStore,
}

impl RateLimitState {
    pub fn new() -> Self {
        Self {
            login_limiter: Arc::new(RateLimiter::direct(Quota::per_minute(5))),
            register_limiter: Arc::new(RateLimiter::direct(Quota::per_minute(3))),
            general_limiters: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Clone)]
pub struct JwtSecret(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: String,
}

pub async fn auth_middleware<B>(
    secret: JwtSecret,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let token = req.headers().get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = token.ok_or(AppError::AuthError)?;

    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.0.as_ref()),
        &Validation::default(),
    ).map_err(|_| AppError::AuthError)?;

    if decoded.claims.exp < Utc::now().timestamp() as usize {
        return Err(AppError::AuthError);
    }

    req.extensions_mut().insert(decoded.claims);
    Ok(next.run(req).await)
}

pub async fn login_rate_limit<B>(
    state: RateLimitState,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    if state.login_limiter.check().is_err() {
        return Err(AppError::RateLimited);
    }
    Ok(next.run(req).await)
}

pub async fn register_rate_limit<B>(
    state: RateLimitState,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    if state.register_limiter.check().is_err() {
        return Err(AppError::RateLimited);
    }
    Ok(next.run(req).await)
}

pub async fn general_rate_limit<B>(
    state: RateLimitState,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let claims = req.extensions().get::<Claims>();
    let key = claims.map(|c| c.sub.clone()).unwrap_or_else(|| {
        req.peer_addr().map(|a| a.to_string()).unwrap_or("unknown".to_string())
    });

    let mut limiters = state.general_limiters.write().await;
    let limiter = limiters.entry(key).or_insert_with(|| {
        RateLimiter::direct(Quota::per_minute(30))
    });

    if limiter.check().is_err() {
        return Err(AppError::RateLimited);
    }

    Ok(next.run(req).await)
}
```

### 前端文件

**package.json**
```json
{
  "name": "{{project_name}}-frontend",
  "private": true,
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "axios": "^1.6.2",
    "lucide-react": "^0.294.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.20.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.37",
    "@types/react-dom": "^18.2.15",
    "@vitejs/plugin-react": "^4.2.0",
    "terser": "^5.47.1",
    "typescript": "^5.2.2",
    "vite": "^5.0.0"
  }
}
```

**vite.config.ts**
```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
    plugins: [react()],
    server: {
        port: 5173,
        proxy: {
            '/api': {
                target: 'http://localhost:{{server_port}}',
                changeOrigin: true,
            },
        },
    },
    build: {
        rollupOptions: {
            output: {
                entryFileNames: 'js/[name]-[hash].js',
                chunkFileNames: 'js/[name]-[hash].js',
                assetFileNames: (assetInfo) => {
                    const info = assetInfo.name.split('.');
                    const ext = info[info.length - 1];
                    if (ext === 'css') {
                        return 'css/[name]-[hash][extname]';
                    }
                    if (/\.(png|jpe?g|gif|svg|webp|ico)$/.test(assetInfo.name)) {
                        return 'images/[name]-[hash][extname]';
                    }
                    if (/\.(woff2?|eot|ttf|otf)$/.test(assetInfo.name)) {
                        return 'fonts/[name]-[hash][extname]';
                    }
                    return 'assets/[name]-[hash][extname]';
                },
                manualChunks: {
                    'react-vendor': ['react', 'react-dom'],
                    'router': ['react-router-dom'],
                    'lucide': ['lucide-react'],
                    'axios': ['axios'],
                },
            },
        },
        manifest: true,
        emptyOutDir: true,
        minify: 'terser',
        terserOptions: {
            compress: {
                drop_console: true,
                drop_debugger: true,
            },
        },
        assetsInlineLimit: 4096,
    },
});
```

**src/api/client.ts**
```typescript
import axios from 'axios';

const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      window.location.href = '/login';
    }
    throw error;
  }
);

export default api;
```

**src/errors/errorCodes.ts**
```typescript
export const errorCodes: Record<string, string> = {
  ERR_UNAUTHORIZED: '认证失败，请重新登录',
  ERR_FORBIDDEN: '权限不足',
  ERR_TOO_MANY_REQUESTS: '请求过于频繁，请稍后再试',
  ERR_ALREADY_EXISTS: '数据已存在',
  ERR_NOT_FOUND: '数据不存在',
  ERR_BAD_REQUEST: '请求参数错误',
  ERR_DATABASE_ERROR: '数据库错误',
  ERR_INTERNAL_ERROR: '服务器内部错误',
  ERR_USER_EXISTS: '用户已存在',
};

export const getErrorMessage = (code: string): string => {
  return errorCodes[code] || '未知错误';
};
```

## 使用方法

### 开发环境

1. **启动数据库**
```bash
docker-compose -f docker-compose.dev.yml up -d
```

2. **配置环境变量**
```bash
cd backend
cp .env.example .env
# 编辑 .env 文件
```

3. **启动后端**
```bash
cd backend
cargo run
```

4. **启动前端**
```bash
cd frontend
npm install
npm run dev
```

### 生产环境

**使用 Docker**
```bash
docker-compose up -d
```

**手动部署**
```bash
cd backend && cargo build --release
cd frontend && npm install && npm run build
```

## 错误码映射

| 错误码 | 描述 |
|--------|------|
| ERR_UNAUTHORIZED | 认证失败，请重新登录 |
| ERR_FORBIDDEN | 权限不足 |
| ERR_TOO_MANY_REQUESTS | 请求过于频繁，请稍后再试 |
| ERR_ALREADY_EXISTS | 数据已存在 |
| ERR_NOT_FOUND | 数据不存在 |
| ERR_BAD_REQUEST | 请求参数错误 |
| ERR_DATABASE_ERROR | 数据库错误 |
| ERR_INTERNAL_ERROR | 服务器内部错误 |

## API 接口

| 接口 | 方法 | 描述 |
|------|------|------|
| `/api/auth/register` | POST | 用户注册 |
| `/api/auth/login` | POST | 用户登录 |
| `/api/auth/me` | POST | 获取当前用户 |
| `/api/health` | GET | 健康检查 |

## 许可证

MIT
