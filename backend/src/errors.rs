use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Too Many Requests: {0}")]
    TooManyRequests(String),

    #[error("Database Error: {0}")]
    Database(String),

    #[error("Internal Error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "ERR_NOT_FOUND",
            AppError::Unauthorized(_) => "ERR_UNAUTHORIZED",
            AppError::Forbidden(_) => "ERR_FORBIDDEN",
            AppError::BadRequest(msg) => {
                // 根据消息内容返回具体的错误码
                match msg.as_str() {
                    "ERR_EMPTY_USERNAME" => "ERR_EMPTY_USERNAME",
                    "ERR_PASSWORD_TOO_SHORT" => "ERR_PASSWORD_TOO_SHORT",
                    "ERR_USER_EXISTS" => "ERR_USER_EXISTS",
                    "ERR_REFERENCE_NOT_FOUND" => "ERR_REFERENCE_NOT_FOUND",
                    "ERR_CATEGORY_EXISTS" => "ERR_CATEGORY_EXISTS",
                    "ERR_LOCATION_EXISTS" => "ERR_LOCATION_EXISTS",
                    "数据已存在" => "ERR_USER_EXISTS",
                    "引用的数据不存在" => "ERR_REFERENCE_NOT_FOUND",
                    _ => "ERR_BAD_REQUEST",
                }
            }
            AppError::TooManyRequests(_) => "ERR_TOO_MANY_REQUESTS",
            AppError::Database(_) => "ERR_DATABASE",
            AppError::Internal(_) => "ERR_INTERNAL",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let code = self.code();
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::BadRequest(msg) => {
                // 如果消息是错误码，返回友好提示
                let friendly_msg = match msg.as_str() {
                    "ERR_EMPTY_USERNAME" => "用户名不能为空",
                    "ERR_PASSWORD_TOO_SHORT" => "密码长度不能少于 6 位",
                    "ERR_USER_EXISTS" => "该用户名已被注册",
                    "ERR_REFERENCE_NOT_FOUND" => "引用的数据不存在",
                    "ERR_CATEGORY_EXISTS" => "分类名称已存在",
                    "ERR_LOCATION_EXISTS" => "地点名称已存在",
                    "数据已存在" => "该用户名已被注册",
                    "引用的数据不存在" => "引用的数据不存在",
                    _ => &msg,
                };
                (StatusCode::BAD_REQUEST, friendly_msg.to_string())
            }
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库错误".to_string())
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "服务器内部错误".to_string(),
                )
            }
        };

        let body = axum::Json(json!({
            "code": code,
            "message": message,
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("资源不存在".to_string()),
            sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23505") => {
                // 唯一约束冲突
                AppError::BadRequest("ERR_USER_EXISTS".to_string())
            }
            sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23503") => {
                // 外键约束冲突
                AppError::BadRequest("ERR_REFERENCE_NOT_FOUND".to_string())
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
