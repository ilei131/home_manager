use chrono::NaiveDate;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// 用户角色
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

// ============================================================
// 用户
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            role: u.role,
            created_at: u.created_at,
        }
    }
}

// ============================================================
// 分类
// ============================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: i32,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CategoryWithFlag {
    pub id: i32,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub created_at: NaiveDateTime,
    #[serde(default)]
    pub is_system: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: String,
}

// ============================================================
// 存放地点
// ============================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Location {
    pub id: i32,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LocationWithFlag {
    pub id: i32,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub created_at: NaiveDateTime,
    #[serde(default)]
    pub is_system: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequest {
    pub name: String,
}

// ============================================================
// 物品
// ============================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub category_id: i32,
    pub location_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemWithDetailsRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub category_id: i32,
    pub category_name: String,
    pub location_id: i32,
    pub location_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemWithDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub category_id: i32,
    pub category_name: String,
    pub location_id: i32,
    pub location_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub batches: Vec<Batch>,
}

impl From<ItemWithDetailsRow> for ItemWithDetails {
    fn from(row: ItemWithDetailsRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            category_id: row.category_id,
            category_name: row.category_name,
            location_id: row.location_id,
            location_name: row.location_name,
            created_at: row.created_at,
            updated_at: row.updated_at,
            batches: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
    pub category_id: i32,
    pub location_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemRequest {
    pub name: Option<String>,
    pub category_id: Option<i32>,
    pub location_id: Option<i32>,
}

// ============================================================
// 分页
// ============================================================

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

// ============================================================
// 批次
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Batch {
    pub id: Uuid,
    pub item_id: Uuid,
    pub quantity: i32,
    pub expiry_date: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateBatchRequest {
    pub quantity: i32,
    pub expiry_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBatchRequest {
    pub quantity: Option<i32>,
    pub expiry_date: Option<Option<NaiveDate>>,
}

// ============================================================
// 统计信息
// ============================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemStats {
    pub total_users: i64,
    pub total_items: i64,
    pub total_categories: i64,
    pub total_locations: i64,
    pub total_batches: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserStats {
    pub total_items: i64,
    pub total_batches: i64,
    pub total_quantity: i64,
    pub expired_batches: i64,
}

// ============================================================
// 认证上下文（注入到请求扩展中）
// ============================================================

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub username: String,
    pub role: UserRole,
}
