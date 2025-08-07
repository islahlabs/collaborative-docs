use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Document {
    pub id: String,
    pub content: String,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DocumentHistory {
    #[schema(value_type = String)]
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDocumentResponse {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateDocumentRequest {
    #[validate(length(min = 0, max = 100000, message = "Content must be between 0 and 100,000 characters"))]
    pub content: String,
}

// User and Authentication Models
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    #[schema(value_type = String)]
    pub id: Uuid,
    pub email: String,
    pub role_id: i32,
    pub role_name: String,
    pub is_active: bool,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SignupRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters long"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub role_id: i32,
    pub role_name: String,
    pub exp: i64, // expiration time
    pub iat: i64, // issued at
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRoleRequest {
    #[validate(length(min = 1, message = "Role name cannot be empty"))]
    pub role_name: String,
}

impl Document {
    pub fn new(id: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}

impl DocumentHistory {
    pub fn new(content: String, ip_address: String) -> Self {
        Self {
            timestamp: Utc::now(),
            ip_address,
            content,
        }
    }
} 