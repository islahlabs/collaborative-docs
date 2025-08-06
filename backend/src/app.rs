use axum::{
    http::{HeaderValue, Method},
    routing::{get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    config::AppConfig,
    database::Database,
    handlers::{
        create_document, get_document, get_document_history, get_document_stats,
        search_documents, update_document,
    },
};

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
        .allow_headers(Any);

    // Create router with all routes
    Router::new()
        .route("/api/doc", post(create_document))
        .route("/api/doc/{id}", get(get_document))
        .route("/api/doc/{id}", put(update_document))
        .route("/api/doc/{id}/history", get(get_document_history))
        .route("/api/doc/{id}/stats", get(get_document_stats))
        .route("/api/search", get(search_documents))
        .layer(cors)
        .with_state(database)
}

/// Create a test application for testing purposes
#[cfg(test)]
pub fn create_test_app(database: Database) -> Router {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers(Any);

    Router::new()
        .route("/api/doc", post(create_document))
        .route("/api/doc/{id}", get(get_document))
        .route("/api/doc/{id}", put(update_document))
        .route("/api/doc/{id}/history", get(get_document_history))
        .layer(cors)
        .with_state(database)
} 