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

    /*
    * This function is used to join a document room.
    * It creates a new broadcast channel if the document room does not exist.
    * It also sends a join message to all other users in the document room.
    * It returns a receiver that can be used to receive messages from the document room.
    * 
    * Note that it used to have a deadlock issue, but it was fixed by using a write lock on the rooms HashMap.
    * This is because tx.send() is a blocking operation, and it acquires a lock on the broadcast channel.
    * The write lock is released after the join message is sent, which prevents the deadlock.
    * 
    * Why This Fixes the Deadlock:
    *   Before (Deadlock):
    *    1. Thread A acquires write lock
    *    2. Thread A calls tx.send() (blocks if channel full)
    *    3. Thread B tries to acquire write lock (waits for Thread A)
    *    4. DEADLOCK! Thread A waiting for send, Thread B waiting for lock
    *   After (No Deadlock):
    *    1. Thread A acquires write lock
    *    2. Thread A gets tx and releases lock immediately
    *    3. Thread A calls tx.send() (no lock held)
    *    4. Thread B can acquire lock normally
    *    5. No deadlock!
    *    The key insight is that tx.send() can block, so we must release the lock before calling it.
    */
    pub async fn join_document(&self, document_id: String, user_id: String) -> broadcast::Receiver<WebSocketMessage> {
        // 1. Create a new scope with curly braces
        let (tx, rx) = {
            // 2. Acquire write lock on the rooms HashMap
            let mut rooms = self.document_rooms.write().await;
            
            // 3. Check if this document already has a broadcast channel
            if let Some(tx) = rooms.get(&document_id) {
                // 4a. If it exists, clone the sender and create a new receiver
                (tx.clone(), tx.subscribe())
            } else {
                // 4b. If it doesn't exist, create a new broadcast channel
                let (tx, rx) = broadcast::channel(100);
                // 5. Store the sender in the HashMap
                rooms.insert(document_id.clone(), tx.clone());
                (tx, rx)
            }
        }; // 6. Write lock is released here (end of scope)

        // 7. Send the join message AFTER releasing the lock (prevents deadlock)
        let _ = tx.send(WebSocketMessage::UserJoined { user_id: user_id.clone() });
        
        // 8. Return the receiver
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
    info!("WebSocket upgrade request for document: {}", document_id);
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