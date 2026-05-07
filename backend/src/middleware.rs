use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::auth::verify_token;
use crate::db::models::{AuthUser, UserRole};
use crate::errors::{AppError, AppResult};

// ============================================================
// 限流配置和存储
// ============================================================

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    reset_time: Instant,
}

type RateLimitStore = Arc<RwLock<HashMap<String, RateLimitEntry>>>;

#[derive(Clone)]
pub struct RateLimitState {
    store: RateLimitStore,
    // 登录限流：每分钟5次
    login_limit: u32,
    login_window: Duration,
    // 注册限流：每分钟3次
    register_limit: u32,
    register_window: Duration,
    // 通用限流：每分钟30次
    general_limit: u32,
    general_window: Duration,
}

impl RateLimitState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            login_limit: 5,
            login_window: Duration::from_secs(60),
            register_limit: 5,
            register_window: Duration::from_secs(60),
            general_limit: 30,
            general_window: Duration::from_secs(60),
        }
    }

    async fn check_limit(&self, key: &str, limit: u32, window: Duration) -> Result<(), AppError> {
        let now = Instant::now();
        let mut store = self.store.write().await;

        // 获取或创建限流条目
        let entry = store
            .entry(key.to_string())
            .or_insert_with(|| RateLimitEntry {
                count: 0,
                reset_time: now + window,
            });

        // 检查是否需要重置
        if now >= entry.reset_time {
            entry.count = 0;
            entry.reset_time = now + window;
        }

        // 检查是否超限
        if entry.count >= limit {
            return Err(AppError::TooManyRequests(
                "请求过于频繁，请稍后再试".to_string(),
            ));
        }

        // 增加计数
        entry.count += 1;
        Ok(())
    }
}

// ============================================================
// 限流中间件实现
// ============================================================

pub async fn login_rate_limit(
    axum::extract::State(state): axum::extract::State<RateLimitState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&req);
    let key = format!("login:{}", ip);

    state
        .check_limit(&key, state.login_limit, state.login_window)
        .await?;
    Ok(next.run(req).await)
}

pub async fn register_rate_limit(
    axum::extract::State(state): axum::extract::State<RateLimitState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&req);
    let key = format!("register:{}", ip);

    state
        .check_limit(&key, state.register_limit, state.register_window)
        .await?;
    Ok(next.run(req).await)
}

pub async fn general_rate_limit(
    axum::extract::State(state): axum::extract::State<RateLimitState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 优先使用用户ID作为限流key
    let key = req
        .extensions()
        .get::<AuthUser>()
        .map(|u| format!("user:{}", u.user_id))
        .unwrap_or_else(|| {
            let ip = extract_client_ip(&req);
            format!("ip:{}", ip)
        });

    state
        .check_limit(&key, state.general_limit, state.general_window)
        .await?;
    Ok(next.run(req).await)
}

// 从请求中提取客户端IP
fn extract_client_ip(req: &Request) -> String {
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or_else(|| {
            req.headers()
                .get("X-Real-IP")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown")
        })
        .trim()
        .to_string()
}

// ============================================================
// JWT Secret 作为 State 传递给中间件
// ============================================================

#[derive(Clone)]
pub struct JwtSecret(pub String);

// ============================================================
// 认证中间件：从 Authorization header 提取并验证 JWT
// ============================================================

pub async fn auth_middleware(
    axum::extract::State(secret): axum::extract::State<JwtSecret>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("缺少 Authorization header".to_string()))?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::Unauthorized("无效的 Authorization 格式，需要 Bearer token".to_string())
    })?;

    let claims = verify_token(token, &secret.0)?;

    let user_id: Uuid = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Token 中的用户 ID 格式无效".to_string()))?;

    let role = match claims.role.as_str() {
        "admin" => UserRole::Admin,
        "user" => UserRole::User,
        _ => UserRole::User,
    };

    let auth_user = AuthUser {
        user_id,
        username: claims.username,
        role,
    };

    // 将认证用户信息注入请求扩展
    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

// ============================================================
// 管理员权限检查
// ============================================================

pub fn require_admin(auth_user: &AuthUser) -> AppResult<()> {
    if auth_user.role != UserRole::Admin {
        return Err(AppError::Forbidden("需要管理员权限".to_string()));
    }
    Ok(())
}
