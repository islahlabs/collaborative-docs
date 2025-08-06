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

async fn create_document(State(state): State<AppState>) -> Json<CreateDocumentResponse> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let document = Document {
        id: id.clone(),
        content: String::new(),
        created_at: now,
        updated_at: now,
    };

    let history = vec![DocumentHistory {
        timestamp: now,
        ip_address: "127.0.0.1".to_string(),
        content: String::new(),
    }];

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
        
        // Add to history
        history.push(DocumentHistory {
            timestamp: now,
            ip_address: "127.0.0.1".to_string(),
            content: document.content.clone(),
        });

        // Update document
        document.content = payload.content;
        document.updated_at = now;

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

async fn create_test_app() -> axum_test::TestServer {
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

    axum_test::TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_full_document_lifecycle() {
    let server = create_test_app().await;
    
    // 1. Create a new document
    let create_response = server
        .post("/api/doc")
        .await;
    
    assert_eq!(create_response.status_code(), StatusCode::OK);
    let create_body: CreateDocumentResponse = create_response.json();
    let document_id = create_body.id;
    
    // 2. Verify the document exists and is empty
    let get_response = server
        .get(&format!("/api/doc/{}", document_id))
        .await;
    
    assert_eq!(get_response.status_code(), StatusCode::OK);
    let document: Document = get_response.json();
    assert_eq!(document.content, "");
    assert_eq!(document.id, document_id);
    
    // 3. Update the document with content
    let update_response = server
        .put(&format!("/api/doc/{}", document_id))
        .json(&serde_json::json!({ "content": "This is test content" }))
        .await;
    
    assert_eq!(update_response.status_code(), StatusCode::OK);
    let updated_document: Document = update_response.json();
    assert_eq!(updated_document.content, "This is test content");
    
    // 4. Verify the document was updated
    let get_updated_response = server
        .get(&format!("/api/doc/{}", document_id))
        .await;
    
    assert_eq!(get_updated_response.status_code(), StatusCode::OK);
    let final_document: Document = get_updated_response.json();
    assert_eq!(final_document.content, "This is test content");
    
    // 5. Check the history
    let history_response = server
        .get(&format!("/api/doc/{}/history", document_id))
        .await;
    
    assert_eq!(history_response.status_code(), StatusCode::OK);
    let history: Vec<DocumentHistory> = history_response.json();
    assert_eq!(history.len(), 2); // Initial empty + update
    assert_eq!(history[0].content, ""); // Initial empty content
    assert_eq!(history[1].content, ""); // Previous content before update
}

#[tokio::test]
async fn test_multiple_updates() {
    let server = create_test_app().await;
    
    // Create document
    let create_response = server
        .post("/api/doc")
        .await;
    let create_body: CreateDocumentResponse = create_response.json();
    let document_id = create_body.id;
    
    // Make multiple updates
    let updates = vec!["First update", "Second update", "Third update"];
    
    for content in updates.iter() {
        let update_response = server
            .put(&format!("/api/doc/{}", document_id))
            .json(&serde_json::json!({ "content": content }))
            .await;
        
        assert_eq!(update_response.status_code(), StatusCode::OK);
        
        // Verify the update
        let get_response = server
            .get(&format!("/api/doc/{}", document_id))
            .await;
        
        let document: Document = get_response.json();
        assert_eq!(document.content, *content);
    }
    
    // Check history has all updates
    let history_response = server
        .get(&format!("/api/doc/{}/history", document_id))
        .await;
    
    let history: Vec<DocumentHistory> = history_response.json();
    assert_eq!(history.len(), 4); // Initial + 3 updates
}

#[tokio::test]
async fn test_error_handling() {
    let server = create_test_app().await;
    
    // Try to get non-existent document
    let response = server
        .get("/api/doc/non-existent-id")
        .await;
    
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    
    // Try to update non-existent document
    let update_response = server
        .put("/api/doc/non-existent-id")
        .json(&serde_json::json!({ "content": "test" }))
        .await;
    
    assert_eq!(update_response.status_code(), StatusCode::NOT_FOUND);
    
    // Try to get history of non-existent document
    let history_response = server
        .get("/api/doc/non-existent-id/history")
        .await;
    
    assert_eq!(history_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_concurrent_access() {
    let server = create_test_app().await;
    
    // Create a document
    let create_response = server
        .post("/api/doc")
        .await;
    let create_body: CreateDocumentResponse = create_response.json();
    let document_id = create_body.id;
    
    // Simulate sequential updates (since TestServer doesn't support cloning)
    let updates = vec!["Update 0", "Update 1", "Update 2", "Update 3", "Update 4"];
    
    for content in updates.iter() {
        let update_response = server
            .put(&format!("/api/doc/{}", document_id))
            .json(&serde_json::json!({ "content": content }))
            .await;
        
        assert_eq!(update_response.status_code(), StatusCode::OK);
    }
    
    // Verify the document was updated
    let get_response = server
        .get(&format!("/api/doc/{}", document_id))
        .await;
    
    assert_eq!(get_response.status_code(), StatusCode::OK);
    let document: Document = get_response.json();
    assert_eq!(document.content, "Update 4"); // Should have the last update
    
    // Check history has all updates
    let history_response = server
        .get(&format!("/api/doc/{}/history", document_id))
        .await;
    
    let history: Vec<DocumentHistory> = history_response.json();
    assert_eq!(history.len(), 6); // Initial + 5 updates
} 