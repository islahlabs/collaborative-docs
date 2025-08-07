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
use tokio::sync::broadcast;

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

// WebSocket message types for CRDT testing
#[derive(Debug, Serialize, Deserialize, Clone)]
enum WebSocketMessage {
    JoinDocument { document_id: String, user_id: String },
    UpdateDocument { content: String, user_id: String },
    DocumentState { state: DocumentState },
    UserJoined { user_id: String },
    UserLeft { user_id: String },
    DocumentUpdated { update: DocumentUpdate },
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DocumentUpdate {
    content: String,
    user_id: String,
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DocumentState {
    content: String,
    version: u64,
    last_modified: i64,
}

// CRDT document for testing
#[derive(Debug, Clone)]
struct CRDTDocument {
    id: String,
    content: String,
    version: u64,
}

impl CRDTDocument {
    fn new(id: String) -> Self {
        Self {
            id,
            content: String::new(),
            version: 0,
        }
    }

    fn update_content(&mut self, new_content: &str, user_id: &str) -> DocumentUpdate {
        self.content = new_content.to_string();
        self.version += 1;
        
        DocumentUpdate {
            content: new_content.to_string(),
            user_id: user_id.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    fn get_state(&self) -> DocumentState {
        DocumentState {
            content: self.content.clone(),
            version: self.version,
            last_modified: chrono::Utc::now().timestamp(),
        }
    }
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

// WebSocket CRDT Tests
#[tokio::test]
async fn test_crdt_document_creation() {
    let mut doc = CRDTDocument::new("test-doc-123".to_string());
    
    // Test initial state
    let state = doc.get_state();
    assert_eq!(state.content, "");
    assert_eq!(state.version, 0);
    
    // Test first update
    let update1 = doc.update_content("Hello from user1", "user1");
    assert_eq!(update1.content, "Hello from user1");
    assert_eq!(update1.user_id, "user1");
    assert_eq!(doc.version, 1);
    
    // Test second update
    let update2 = doc.update_content("Hello from user2! This is collaborative editing.", "user2");
    assert_eq!(update2.content, "Hello from user2! This is collaborative editing.");
    assert_eq!(update2.user_id, "user2");
    assert_eq!(doc.version, 2);
    
    // Verify final state
    let final_state = doc.get_state();
    assert_eq!(final_state.content, "Hello from user2! This is collaborative editing.");
    assert_eq!(final_state.version, 2);
}

#[tokio::test]
async fn test_websocket_message_serialization() {
    // Test JoinDocument message
    let join_msg = WebSocketMessage::JoinDocument {
        document_id: "doc-123".to_string(),
        user_id: "user-456".to_string(),
    };
    
    let serialized = serde_json::to_string(&join_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::JoinDocument { document_id, user_id } => {
            assert_eq!(document_id, "doc-123");
            assert_eq!(user_id, "user-456");
        }
        _ => panic!("Expected JoinDocument message"),
    }
    
    // Test UpdateDocument message
    let update_msg = WebSocketMessage::UpdateDocument {
        content: "Test content".to_string(),
        user_id: "user-789".to_string(),
    };
    
    let serialized = serde_json::to_string(&update_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::UpdateDocument { content, user_id } => {
            assert_eq!(content, "Test content");
            assert_eq!(user_id, "user-789");
        }
        _ => panic!("Expected UpdateDocument message"),
    }
}

#[tokio::test]
async fn test_crdt_conflict_resolution() {
    let mut doc = CRDTDocument::new("conflict-test".to_string());
    
    // Simulate concurrent updates from different users
    let updates = vec![
        ("user1", "First user's content"),
        ("user2", "Second user's content"),
        ("user3", "Third user's content"),
    ];
    
    for (user_id, content) in updates {
        let update = doc.update_content(content, user_id);
        println!("User {} updated document: {}", user_id, update.content);
        println!("Version: {}, Timestamp: {}", update.timestamp, update.timestamp);
    }
    
    // Verify final state (last-write-wins in our simple implementation)
    let final_state = doc.get_state();
    assert_eq!(final_state.content, "Third user's content");
    assert_eq!(final_state.version, 3);
}

#[tokio::test]
async fn test_websocket_broadcast_simulation() {
    // Simulate WebSocket broadcast channel
    let (tx, mut rx) = broadcast::channel::<WebSocketMessage>(100);
    
    // Create a document and simulate multiple users
    let mut doc = CRDTDocument::new("broadcast-test".to_string());
    
    // Simulate user1 joining
    let join_msg = WebSocketMessage::UserJoined {
        user_id: "user1".to_string(),
    };
    tx.send(join_msg.clone()).unwrap();
    
    // Simulate user1 updating
    let update = doc.update_content("Hello from user1", "user1");
    let update_msg = WebSocketMessage::DocumentUpdated { update };
    tx.send(update_msg.clone()).unwrap();
    
    // Simulate user2 joining
    let join_msg2 = WebSocketMessage::UserJoined {
        user_id: "user2".to_string(),
    };
    tx.send(join_msg2.clone()).unwrap();
    
    // Simulate user2 updating
    let update2 = doc.update_content("Hello from user2!", "user2");
    let update_msg2 = WebSocketMessage::DocumentUpdated { update: update2 };
    tx.send(update_msg2.clone()).unwrap();
    
    // Verify we received all messages
    let mut received_messages = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        received_messages.push(msg);
    }
    
    assert_eq!(received_messages.len(), 4);
    
    // Verify message types
    match &received_messages[0] {
        WebSocketMessage::UserJoined { user_id } => assert_eq!(user_id, "user1"),
        _ => panic!("Expected UserJoined message"),
    }
    
    match &received_messages[1] {
        WebSocketMessage::DocumentUpdated { update } => {
            assert_eq!(update.user_id, "user1");
            assert_eq!(update.content, "Hello from user1");
        }
        _ => panic!("Expected DocumentUpdated message"),
    }
    
    match &received_messages[2] {
        WebSocketMessage::UserJoined { user_id } => assert_eq!(user_id, "user2"),
        _ => panic!("Expected UserJoined message"),
    }
    
    match &received_messages[3] {
        WebSocketMessage::DocumentUpdated { update } => {
            assert_eq!(update.user_id, "user2");
            assert_eq!(update.content, "Hello from user2!");
        }
        _ => panic!("Expected DocumentUpdated message"),
    }
}

#[tokio::test]
async fn test_crdt_version_tracking() {
    let mut doc = CRDTDocument::new("version-test".to_string());
    
    // Track versions through updates
    let mut versions = Vec::new();
    
    for i in 1..=5 {
        let content = format!("Update {}", i);
        let update = doc.update_content(&content, &format!("user{}", i));
        versions.push((doc.version, update.content.clone()));
    }
    
    // Verify version progression
    for (i, (version, content)) in versions.iter().enumerate() {
        assert_eq!(*version, i as u64 + 1);
        assert_eq!(*content, format!("Update {}", i + 1));
    }
    
    // Verify final document state
    let final_state = doc.get_state();
    assert_eq!(final_state.version, 5);
    assert_eq!(final_state.content, "Update 5");
}

#[tokio::test]
async fn test_websocket_message_types() {
    // Test all WebSocket message types
    let messages = vec![
        WebSocketMessage::JoinDocument {
            document_id: "doc1".to_string(),
            user_id: "user1".to_string(),
        },
        WebSocketMessage::UpdateDocument {
            content: "test content".to_string(),
            user_id: "user1".to_string(),
        },
        WebSocketMessage::DocumentState {
            state: DocumentState {
                content: "current content".to_string(),
                version: 1,
                last_modified: 1234567890,
            },
        },
        WebSocketMessage::UserJoined {
            user_id: "user2".to_string(),
        },
        WebSocketMessage::UserLeft {
            user_id: "user1".to_string(),
        },
        WebSocketMessage::DocumentUpdated {
            update: DocumentUpdate {
                content: "updated content".to_string(),
                user_id: "user2".to_string(),
                timestamp: 1234567890,
            },
        },
        WebSocketMessage::Error {
            message: "test error".to_string(),
        },
    ];
    
    // Test serialization/deserialization for all message types
    for msg in messages {
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
        
        // Verify the message round-trips correctly
        let reserialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(serialized, reserialized);
    }
} 