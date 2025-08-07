use axum::{
    http::{HeaderValue, Method, HeaderName},
    routing::{get, post, put},
    Router,
    middleware,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

use crate::{
    config::AppConfig,
    database::Database,
    auth::auth_middleware,
    handlers::{
        create_document, get_document, get_document_history, get_document_stats,
        search_documents, update_document, get_document_crdt_state, apply_crdt_update,
        signup, login, create_document_protected, update_user_role,
    },
    websocket::{websocket_handler, websocket_info_handler, WebSocketManager},
};

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub ws_manager: Arc<WebSocketManager>,
}

/// Create the application router with all routes and middleware
pub fn create_app(database: Database, config: &AppConfig) -> Router {
    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(config.cors.allowed_origins.iter().map(|origin| {
            origin.parse::<HeaderValue>().unwrap_or_else(|_| {
                "http://localhost:5173".parse::<HeaderValue>().unwrap()
            })
        }).collect::<Vec<_>>())
        .allow_methods(config.cors.allowed_methods.iter().map(|method| {
            method.parse::<Method>().unwrap_or(Method::GET)
        }).collect::<Vec<_>>())
        .allow_headers([
            "content-type".parse::<HeaderName>().unwrap(),
            "authorization".parse::<HeaderName>().unwrap(),
            "accept".parse::<HeaderName>().unwrap(),
            "origin".parse::<HeaderName>().unwrap(),
            "x-requested-with".parse::<HeaderName>().unwrap(),
        ]);

    // Create WebSocket manager
    let ws_manager = Arc::new(WebSocketManager::new());

    // Create combined state
    let state = AppState {
        database,
        ws_manager,
    };

    // Create router with all routes
    let public_routes = Router::new()
        // Public authentication routes
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        // Public document routes (no authentication required)
        .route("/api/doc/{id}", get(get_document))
        .route("/api/doc/{id}", put(update_document))
        .route("/api/doc/{id}/history", get(get_document_history))
        .route("/api/doc/{id}/stats", get(get_document_stats))
        .route("/api/search", get(search_documents))
        // CRDT routes for real-time collaboration
        .route("/api/doc/{id}/crdt/state", get(get_document_crdt_state))
        .route("/api/doc/{id}/crdt/update", post(apply_crdt_update))
        // WebSocket routes
        .route("/ws/doc/{document_id}", get(websocket_handler))
        .route("/ws/info/{document_id}", get(websocket_info_handler));

    let protected_routes = Router::new()
        // Protected document routes (require authentication)
        .route("/api/doc", post(create_document_protected))
        // Admin routes (require admin role)
        .route("/api/admin/users/{user_id}/role", put(update_user_role))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    public_routes
        .merge(protected_routes)
        .layer(cors)
        .with_state(state)
}

/// Create a test application for testing purposes
#[cfg(test)]
pub fn create_test_app(database: Database) -> Router {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers([
            "content-type".parse::<HeaderName>().unwrap(),
            "authorization".parse::<HeaderName>().unwrap(),
            "accept".parse::<HeaderName>().unwrap(),
            "origin".parse::<HeaderName>().unwrap(),
            "x-requested-with".parse::<HeaderName>().unwrap(),
        ]);

    let ws_manager = Arc::new(WebSocketManager::new());
    let state = AppState {
        database,
        ws_manager,
    };

    Router::new()
        .route("/api/doc", post(create_document))
        .route("/api/doc/{id}", get(get_document))
        .route("/api/doc/{id}", put(update_document))
        .route("/api/doc/{id}/history", get(get_document_history))
        .layer(cors)
        .with_state(state)
} 