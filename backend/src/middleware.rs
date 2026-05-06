use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use crate::db::auth::verify_token;
use crate::db::models::{AuthUser, UserRole};
use crate::errors::{AppError, AppResult};

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

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
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
