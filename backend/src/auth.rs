use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{Claims, User},
};

const JWT_SECRET: &[u8] = b"your-secret-key-change-in-production";
const JWT_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub role_id: i32,
    pub role_name: String,
}

pub async fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, DEFAULT_COST)
        .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))
}

pub async fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash)
        .map_err(|e| AppError::InternalError(format!("Password verification failed: {}", e)))
}

pub fn create_jwt_token(user: &User) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now + Duration::hours(JWT_EXPIRATION_HOURS);
    
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role_id: user.role_id,
        role_name: user.role_name.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|e| AppError::InternalError(format!("JWT encoding failed: {}", e)))
}

pub fn verify_jwt_token(token: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map_err(|e| AppError::AuthenticationError(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer ") {
                Some(auth_str[7..].to_string())
            } else {
                None
            }
        });

    let token = auth_header.ok_or_else(|| {
        AppError::AuthenticationError("Missing authorization header".to_string())
    })?;

    let claims = verify_jwt_token(&token)?;
    
    // Check if token is expired
    let now = Utc::now().timestamp();
    if claims.exp < now {
        return Err(AppError::AuthenticationError("Token expired".to_string()));
    }

    let authenticated_user = AuthenticatedUser {
        user_id: Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::AuthenticationError("Invalid user ID in token".to_string()))?,
        email: claims.email,
        role_id: claims.role_id,
        role_name: claims.role_name,
    };

    // Insert the authenticated user into the request extensions
    request.extensions_mut().insert(authenticated_user);

    Ok(next.run(request).await)
}

pub fn require_role(required_role: &str) -> impl Fn(&AuthenticatedUser) -> AppResult<()> {
    let required_role = required_role.to_string();
    move |user: &AuthenticatedUser| {
        if user.role_name == required_role || user.role_name == "admin" {
            Ok(())
        } else {
            Err(AppError::AuthorizationError(format!(
                "Insufficient permissions. Required role: {}, User role: {}",
                required_role, user.role_name
            )))
        }
    }
} 