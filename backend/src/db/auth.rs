use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{LoginResponse, RegisterRequest, User, UserResponse};
use crate::errors::{AppError, AppResult};

// ============================================================
// JWT Claims
// ============================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub role: String,
    pub exp: usize, // 过期时间戳
    pub iat: usize, // 签发时间戳
}

pub fn generate_token(user: &User, jwt_secret: &str) -> AppResult<String> {
    let now = Utc::now();
    let expire = now + chrono::Duration::hours(24);

    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: match user.role {
            super::models::UserRole::Admin => "admin".to_string(),
            super::models::UserRole::User => "user".to_string(),
        },
        exp: expire.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("生成 token 失败: {}", e)))
}

pub fn verify_token(token: &str, jwt_secret: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::Unauthorized("Token 已过期".to_string())
        }
        _ => AppError::Unauthorized("无效的 Token".to_string()),
    })
}

// ============================================================
// 注册
// ============================================================

pub async fn register(
    pool: &PgPool,
    req: RegisterRequest,
    jwt_secret: &str,
) -> AppResult<LoginResponse> {
    if req.username.trim().is_empty() {
        return Err(AppError::BadRequest("ERR_EMPTY_USERNAME".to_string()));
    }
    if req.password.len() < 6 {
        return Err(AppError::BadRequest("ERR_PASSWORD_TOO_SHORT".to_string()));
    }

    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("密码加密失败: {}", e)))?;

    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (username, password_hash, role)
           VALUES ($1, $2, 'user')
           RETURNING id, username, password_hash, role, created_at"#,
    )
    .bind(&req.username)
    .bind(&password_hash)
    .fetch_one(pool)
    .await?;

    let token = generate_token(&user, jwt_secret)?;
    let user_response = UserResponse::from(user);

    Ok(LoginResponse {
        token,
        user: user_response,
    })
}

// ============================================================
// 登录
// ============================================================

pub async fn login(
    pool: &PgPool,
    username: &str,
    password: &str,
    jwt_secret: &str,
) -> AppResult<LoginResponse> {
    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, username, password_hash, role, created_at
           FROM users WHERE username = $1"#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("ERR_INVALID_CREDENTIALS".to_string()))?;

    let valid = bcrypt::verify(password, &user.password_hash)
        .map_err(|e| AppError::Internal(format!("密码验证失败: {}", e)))?;

    if !valid {
        return Err(AppError::Unauthorized(
            "ERR_INVALID_CREDENTIALS".to_string(),
        ));
    }

    let token = generate_token(&user, jwt_secret)?;
    let user_response = UserResponse::from(user);

    Ok(LoginResponse {
        token,
        user: user_response,
    })
}

// ============================================================
// 获取用户信息
// ============================================================

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> AppResult<User> {
    sqlx::query_as::<_, User>(
        r#"SELECT id, username, password_hash, role, created_at
           FROM users WHERE id = $1"#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("用户不存在".to_string()))
}

// ============================================================
// 获取所有用户列表（管理员权限）
// ============================================================

pub async fn get_all_users(pool: &PgPool) -> AppResult<Vec<UserResponse>> {
    let users = sqlx::query_as::<_, User>(
        r#"SELECT id, username, password_hash, role, created_at
           FROM users ORDER BY created_at DESC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(users.into_iter().map(UserResponse::from).collect())
}

// ============================================================
// 确保系统有默认管理员用户
// ============================================================

pub async fn ensure_admin(pool: &PgPool, default_password: &str) -> AppResult<()> {
    // 检查是否已存在 admin 用户
    let existing = sqlx::query_as::<_, User>(
        r#"SELECT id, username, password_hash, role, created_at
           FROM users WHERE username = 'admin'"#,
    )
    .fetch_optional(pool)
    .await?;

    if existing.is_none() {
        let password_hash = bcrypt::hash(default_password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("密码加密失败: {}", e)))?;

        sqlx::query(
            r#"INSERT INTO users (username, password_hash, role)
               VALUES ('admin', $1, 'admin')"#,
        )
        .bind(&password_hash)
        .execute(pool)
        .await?;

        tracing::info!("默认管理员用户已创建（用户名: admin）");
    }

    Ok(())
}
