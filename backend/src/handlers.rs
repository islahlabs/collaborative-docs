use axum::{
    extract::{Path, State, Extension},
    response::Json,
    http::HeaderMap,
};
use validator::Validate;

use crate::{
    app::AppState,
    auth::{AuthenticatedUser, require_role},
    error::{AppError, AppResult},
    models::{CreateDocumentResponse, Document, DocumentHistory, UpdateDocumentRequest, SignupRequest, LoginRequest, AuthResponse, User, UpdateUserRoleRequest},
    crdt::{DocumentUpdate, DocumentState},
    utils::{extract_client_ip_from_headers},
};

/// Create a new document
pub async fn create_document(
    State(state): State<AppState>,
) -> AppResult<Json<CreateDocumentResponse>> {
    let id = state.database.create_document().await?;
    Ok(Json(CreateDocumentResponse { id }))
}

/// Get a document by ID
pub async fn get_document(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<Document>> {
    let document = state.database.get_document(&id).await?;
    Ok(Json(document))
}

/// Update a document's content
pub async fn update_document(
    Path(id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateDocumentRequest>,
) -> AppResult<Json<Document>> {
    // Validate input
    payload.validate().map_err(|e| {
        AppError::ValidationError(format!("Validation failed: {}", e))
    })?;

    // Extract IP address from headers (proxy headers or fallback to localhost)
    let ip_address = extract_client_ip_from_headers(&headers);
    
    let document = state.database.update_document(&id, &payload.content, &ip_address).await?;
    Ok(Json(document))
}

/// Get document history
pub async fn get_document_history(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<DocumentHistory>>> {
    let history = state.database.get_document_history(&id).await?;
    Ok(Json(history))
}

/// Get document statistics
pub async fn get_document_stats(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let (history_count, last_updated) = state.database.get_document_stats(&id).await?;
    
    Ok(Json(serde_json::json!({
        "history_count": history_count,
        "last_updated": last_updated
    })))
}

/// Search documents by content
pub async fn search_documents(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<Document>>> {
    let empty_string = String::new();
    let query = params.get("q").unwrap_or(&empty_string);
    
    if query.is_empty() {
        return Err(AppError::ValidationError("Search query 'q' is required".to_string()));
    }
    
    let documents = state.database.search_documents(query).await?;
    Ok(Json(documents))
}

/// CRDT: Get document state (for real-time sync)
pub async fn get_document_crdt_state(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<DocumentState>> {
    let state = state.database.get_document_crdt_state(&id).await?;
    Ok(Json(state))
}

/// CRDT: Apply update from another client
pub async fn apply_crdt_update(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(update): Json<DocumentUpdate>,
) -> AppResult<Json<serde_json::Value>> {
    state.database.apply_crdt_update(&id, &update).await?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Update applied successfully"
    })))
}

// Authentication Handlers
pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Validate input
    payload.validate().map_err(|e| {
        AppError::ValidationError(format!("Validation failed: {}", e))
    })?;

    // Hash password
    let password_hash = crate::auth::hash_password(&payload.password).await?;

    // Create user
    let user = state.database.create_user(&payload, &password_hash).await?;

    // Generate JWT token
    let token = crate::auth::create_jwt_token(&user)?;

    Ok(Json(AuthResponse { token, user }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Validate input
    payload.validate().map_err(|e| {
        AppError::ValidationError(format!("Validation failed: {}", e))
    })?;

    // Get user and password hash
    let user = state.database.authenticate_user(&payload).await?;
    let password_hash = state.database.get_user_password_hash(&payload.email).await?;

    // Verify password
    let is_valid = crate::auth::verify_password(&payload.password, &password_hash).await?;
    if !is_valid {
        return Err(AppError::AuthenticationError("Invalid password".to_string()));
    }

    // Generate JWT token
    let token = crate::auth::create_jwt_token(&user)?;

    Ok(Json(AuthResponse { token, user }))
}

// Protected document creation handler
pub async fn create_document_protected(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> AppResult<Json<CreateDocumentResponse>> {
    // Check if user has permission to create documents
    let check_permission = require_role("document_creator");
    check_permission(&user)?;

    let id = state.database.create_document().await?;
    Ok(Json(CreateDocumentResponse { id }))
}

// Admin handler to update user roles
pub async fn update_user_role(
    Extension(admin_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> AppResult<Json<User>> {
    // Check if user is admin
    let check_permission = require_role("admin");
    check_permission(&admin_user)?;

    // Validate input
    payload.validate().map_err(|e| {
        AppError::ValidationError(format!("Validation failed: {}", e))
    })?;

    // Update user role
    let user = state.database.update_user_role(&user_id, &payload.role_name).await?;
    Ok(Json(user))
} 