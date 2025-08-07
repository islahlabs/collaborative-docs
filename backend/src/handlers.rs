use axum::{
    extract::{Path, State},
    response::Json,
    http::HeaderMap,
};
use validator::Validate;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    models::{CreateDocumentResponse, Document, DocumentHistory, UpdateDocumentRequest},
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