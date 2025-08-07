use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use axum_tws::{WebSocket, WebSocketUpgrade};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
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

// WebSocket handler for real-time CRDT collaboration
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(document_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, document_id, state))
}

async fn handle_socket(socket: WebSocket, document_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Generate a unique user ID for this connection
    let user_id = Uuid::new_v4().to_string();
    info!("WebSocket connection established for document {} by user {}", document_id, user_id);

    // Join the document room
    let mut rx = state.ws_manager.join_document(document_id.clone(), user_id.clone()).await;

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(msg) => {
                    if msg.is_text() {
                        // For now, just log the message
                        info!("Received WebSocket message: {:?}", msg);
                    }
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Handle outgoing messages
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg).unwrap();
            if let Err(e) = sender.send(axum_tws::Message::text(text)).await {
                error!("Failed to send WebSocket message: {}", e);
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = (&mut recv_task) => {
            send_task.abort();
        }
        _ = (&mut send_task) => {
            recv_task.abort();
        }
    }

    // Leave the document room
    state.ws_manager.leave_document(&document_id, &user_id).await;
    info!("WebSocket connection closed for document {} by user {}", document_id, user_id);
}

// HTTP endpoint that returns WebSocket info (for debugging)
pub async fn websocket_info_handler(
    Path(document_id): Path<String>,
    State(_state): State<AppState>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "message": "WebSocket endpoint",
        "document_id": document_id,
        "endpoint": format!("/ws/doc/{}", document_id),
        "protocol": "ws:// or wss://"
    }))
} 