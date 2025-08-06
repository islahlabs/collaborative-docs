use axum::{
    extract::{Path, State},
    http::{HeaderValue, Method, StatusCode},
    response::Json,
    routing::{get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use serde_json::json;

    async fn create_test_app() -> TestServer {
        let state: AppState = Arc::new(RwLock::new(HashMap::new()));
        let cors = CorsLayer::new()
            .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT])
            .allow_headers(Any);

        let app = Router::new()
            .route("/api/doc", post(create_document))
            .route("/api/doc/{id}", get(get_document))
            .route("/api/doc/{id}", put(update_document))
            .route("/api/doc/{id}/history", get(get_document_history))
            .layer(cors)
            .with_state(state);

        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_create_document() {
        let server = create_test_app().await;
        
        let response = server
            .post("/api/doc")
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: CreateDocumentResponse = response.json();
        assert!(!body.id.is_empty());
    }

    #[tokio::test]
    async fn test_get_nonexistent_document() {
        let server = create_test_app().await;
        
        let response = server
            .get("/api/doc/nonexistent-id")
            .await;
        
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_and_get_document() {
        let server = create_test_app().await;
        
        // Create document
        let create_response = server
            .post("/api/doc")
            .await;
        
        assert_eq!(create_response.status_code(), StatusCode::OK);
        let create_body: CreateDocumentResponse = create_response.json();
        
        // Get document
        let get_response = server
            .get(&format!("/api/doc/{}", create_body.id))
            .await;
        
        assert_eq!(get_response.status_code(), StatusCode::OK);
        let document: Document = get_response.json();
        assert_eq!(document.id, create_body.id);
        assert_eq!(document.content, "");
    }

    #[tokio::test]
    async fn test_update_document() {
        let server = create_test_app().await;
        
        // Create document
        let create_response = server
            .post("/api/doc")
            .await;
        let create_body: CreateDocumentResponse = create_response.json();
        
        // Update document
        let update_response = server
            .put(&format!("/api/doc/{}", create_body.id))
            .json(&json!({ "content": "Hello, World!" }))
            .await;
        
        assert_eq!(update_response.status_code(), StatusCode::OK);
        let document: Document = update_response.json();
        assert_eq!(document.content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_get_document_history() {
        let server = create_test_app().await;
        
        // Create document
        let create_response = server
            .post("/api/doc")
            .await;
        let create_body: CreateDocumentResponse = create_response.json();
        
        // Update document
        server
            .put(&format!("/api/doc/{}", create_body.id))
            .json(&json!({ "content": "Updated content" }))
            .await;
        
        // Get history
        let history_response = server
            .get(&format!("/api/doc/{}/history", create_body.id))
            .await;
        
        assert_eq!(history_response.status_code(), StatusCode::OK);
        let history: Vec<DocumentHistory> = history_response.json();
        assert!(!history.is_empty());
        assert_eq!(history.len(), 1); // Only the update (no initial empty entry)
        assert_eq!(history[0].content, "Updated content");
    }

    #[tokio::test]
    async fn test_multiple_updates_create_history() {
        let server = create_test_app().await;
        
        // Create document
        let create_response = server
            .post("/api/doc")
            .await;
        let create_body: CreateDocumentResponse = create_response.json();
        
        // First update
        server
            .put(&format!("/api/doc/{}", create_body.id))
            .json(&json!({ "content": "First version" }))
            .await;
        
        // Second update
        server
            .put(&format!("/api/doc/{}", create_body.id))
            .json(&json!({ "content": "Second version" }))
            .await;
        
        // Third update
        server
            .put(&format!("/api/doc/{}", create_body.id))
            .json(&json!({ "content": "Third version" }))
            .await;
        
        // Get history
        let history_response = server
            .get(&format!("/api/doc/{}/history", create_body.id))
            .await;
        
        assert_eq!(history_response.status_code(), StatusCode::OK);
        let history: Vec<DocumentHistory> = history_response.json();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].content, "First version");
        assert_eq!(history[1].content, "Second version");
        assert_eq!(history[2].content, "Third version");
    }

    #[tokio::test]
    async fn test_history_timestamps_and_ip() {
        let server = create_test_app().await;
        
        // Create document
        let create_response = server
            .post("/api/doc")
            .await;
        let create_body: CreateDocumentResponse = create_response.json();
        
        // Update document
        server
            .put(&format!("/api/doc/{}", create_body.id))
            .json(&json!({ "content": "Test content" }))
            .await;
        
        // Get history
        let history_response = server
            .get(&format!("/api/doc/{}/history", create_body.id))
            .await;
        
        assert_eq!(history_response.status_code(), StatusCode::OK);
        let history: Vec<DocumentHistory> = history_response.json();
        assert_eq!(history.len(), 1);
        
        let entry = &history[0];
        assert_eq!(entry.content, "Test content");
        assert_eq!(entry.ip_address, "127.0.0.1");
        assert!(!entry.timestamp.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_history_for_nonexistent_document() {
        let server = create_test_app().await;
        
        let response = server
            .get("/api/doc/nonexistent-id/history")
            .await;
        
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Document {
    id: String,
    content: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DocumentHistory {
    timestamp: DateTime<Utc>,
    ip_address: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateDocumentResponse {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateDocumentRequest {
    content: String,
}

type AppState = Arc<RwLock<HashMap<String, (Document, Vec<DocumentHistory>)>>>;

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(RwLock::new(HashMap::new()));

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/doc", post(create_document))
        .route("/api/doc/{id}", get(get_document))
        .route("/api/doc/{id}", put(update_document))
        .route("/api/doc/{id}/history", get(get_document_history))
        .layer(cors)
        .with_state(state);

    println!("🚀 Server starting on http://localhost:3000");
    println!("📝 API endpoints:");
    println!("  POST   /api/doc");
    println!("  GET    /api/doc/:id");
    println!("  PUT    /api/doc/:id");
    println!("  GET    /api/doc/:id/history");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_document(State(state): State<AppState>) -> Json<CreateDocumentResponse> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let document = Document {
        id: id.clone(),
        content: String::new(),
        created_at: now,
        updated_at: now,
    };

    let history = vec![]; // Start with empty history, will be populated on first update

    state.write().unwrap().insert(id.clone(), (document, history));

    Json(CreateDocumentResponse { id })
}

async fn get_document(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Document>, StatusCode> {
    let state = state.read().unwrap();
    
    if let Some((document, _)) = state.get(&id) {
        Ok(Json(document.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn update_document(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateDocumentRequest>,
) -> Result<Json<Document>, StatusCode> {
    let mut state = state.write().unwrap();
    
    if let Some((document, history)) = state.get_mut(&id) {
        let now = Utc::now();
        
        // Update document
        document.content = payload.content.clone();
        document.updated_at = now;

        // Add to history with the new content
        history.push(DocumentHistory {
            timestamp: now,
            ip_address: "127.0.0.1".to_string(), // TODO: Extract real IP
            content: payload.content,
        });

        Ok(Json(document.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_document_history(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DocumentHistory>>, StatusCode> {
    let state = state.read().unwrap();
    
    if let Some((_, history)) = state.get(&id) {
        Ok(Json(history.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
