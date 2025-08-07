use collaborative_docs_rs::auth::{hash_password, verify_password, create_jwt_token, verify_jwt_token};
use collaborative_docs_rs::models::User;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_password_hashing() {
    let password = "test_password_123";
    let hash = hash_password(password).await.unwrap();
    
    // Verify the password
    let is_valid = verify_password(password, &hash).await.unwrap();
    assert!(is_valid);
    
    // Verify wrong password fails
    let is_invalid = verify_password("wrong_password", &hash).await.unwrap();
    assert!(!is_invalid);
}

#[tokio::test]
async fn test_jwt_token_creation_and_verification() {
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        role_id: 1,
        role_name: "admin".to_string(),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create token
    let token = create_jwt_token(&user).unwrap();
    
    // Verify token
    let claims = verify_jwt_token(&token).unwrap();
    
    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.email, user.email);
    assert_eq!(claims.role_id, user.role_id);
    assert_eq!(claims.role_name, user.role_name);
}

#[tokio::test]
async fn test_jwt_token_expiration() {
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        role_id: 1,
        role_name: "admin".to_string(),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create token
    let token = create_jwt_token(&user).unwrap();
    
    // Verify token is valid
    let claims = verify_jwt_token(&token).unwrap();
    assert!(claims.exp > Utc::now().timestamp());
}

#[tokio::test]
async fn test_invalid_jwt_token() {
    // Test with invalid token
    let result = verify_jwt_token("invalid.token.here");
    assert!(result.is_err());
} 