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

    #[error("Database Error: {0}")]
    Database(String),

    #[error("Internal Error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "数据库错误".to_string(),
                )
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
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {
                AppError::NotFound("资源不存在".to_string())
            }
            sqlx::Error::Database(ref db_err)
                if db_err.code().as_deref() == Some("23505") =>
            {
                AppError::BadRequest("数据已存在（唯一约束冲突）".to_string())
            }
            sqlx::Error::Database(ref db_err)
                if db_err.code().as_deref() == Some("23503") =>
            {
                AppError::BadRequest("引用的数据不存在（外键约束）".to_string())
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
