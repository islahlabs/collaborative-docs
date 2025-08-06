use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{info, warn, error};
use uuid::Uuid;
use futures_util::{SinkExt, StreamExt};

use crate::{
    app::AppState,
    crdt::{DocumentUpdate, DocumentState},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WebSocketMessage {
    // Client -> Server
    JoinDocument { document_id: String, user_id: String },
    UpdateDocument { content: String, user_id: String },
    
    // Server -> Client
    DocumentState { state: DocumentState },
    UserJoined { user_id: String },
    UserLeft { user_id: String },
    DocumentUpdated { update: DocumentUpdate },
    Error { message: String },
}

#[derive(Debug)]
pub struct WebSocketConnection {
    pub id: String,
    pub user_id: String,
    pub document_id: String,
}

#[derive(Debug)]
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    document_rooms: Arc<RwLock<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            document_rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn join_document(&self, document_id: String, user_id: String) -> broadcast::Receiver<WebSocketMessage> {
        let mut rooms = self.document_rooms.write().await;
        
        let (tx, rx) = if let Some(tx) = rooms.get(&document_id) {
            (tx.clone(), tx.subscribe())
        } else {
            let (tx, rx) = broadcast::channel(100);
            rooms.insert(document_id.clone(), tx.clone());
            (tx, rx)
        };

        // Notify other users that someone joined
        let _ = tx.send(WebSocketMessage::UserJoined { user_id: user_id.clone() });
        
        rx
    }

    pub async fn leave_document(&self, document_id: &str, user_id: &str) {
        let rooms = self.document_rooms.read().await;
        if let Some(tx) = rooms.get(document_id) {
            let _ = tx.send(WebSocketMessage::UserLeft { user_id: user_id.to_string() });
        }
    }

    pub async fn broadcast_update(&self, document_id: &str, update: DocumentUpdate) {
        let rooms = self.document_rooms.read().await;
        if let Some(tx) = rooms.get(document_id) {
            let _ = tx.send(WebSocketMessage::DocumentUpdated { update });
        }
    }

    pub async fn broadcast_state(&self, document_id: &str, state: DocumentState) {
        let rooms = self.document_rooms.read().await;
        if let Some(tx) = rooms.get(document_id) {
            let _ = tx.send(WebSocketMessage::DocumentState { state });
        }
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

// For now, let's create a simple HTTP endpoint that returns WebSocket info
pub async fn websocket_info_handler(
    Path(document_id): Path<String>,
    State(state): State<AppState>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "message": "WebSocket endpoint",
        "document_id": document_id,
        "endpoint": format!("/ws/doc/{}", document_id),
        "protocol": "ws:// or wss://"
    }))
} 