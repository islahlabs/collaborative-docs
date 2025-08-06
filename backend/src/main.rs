mod config;
mod database;
mod error;
mod models;

use axum::{
    extract::{Path, State},
    http::{HeaderValue, Method},
    response::Json,
    routing::{get, post, put},
    Router,
};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use validator::Validate;

use crate::{
    config::AppConfig,
    database::Database,
    error::{AppError, AppResult},
    models::{CreateDocumentResponse, Document, DocumentHistory, UpdateDocumentRequest},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration
    let config = AppConfig::load()?;
    
    // Initialize database
    let database = Database::new(&config.database_url()).await.map_err(|e| {
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    })?;
    
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

    // Create router
    let app = Router::new()
        .route("/api/doc", post(create_document))
        .route("/api/doc/{id}", get(get_document))
        .route("/api/doc/{id}", put(update_document))
        .route("/api/doc/{id}/history", get(get_document_history))
        .route("/api/doc/{id}/stats", get(get_document_stats))
        .route("/api/search", get(search_documents))
        .layer(cors)
        .with_state(database);

    // Parse host address
    let host_ip = if config.server.host == "0.0.0.0" {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    } else {
        config.server.host.parse().unwrap_or_else(|_| {
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        })
    };
    
    let addr = SocketAddr::from((host_ip, config.server.port));
    
    info!("🚀 Server starting on http://{}", addr);
    info!("📝 API endpoints:");
    info!("  POST   /api/doc");
    info!("  GET    /api/doc/:id");
    info!("  PUT    /api/doc/:id");
    info!("  GET    /api/doc/:id/history");
    info!("  GET    /api/doc/:id/stats");
    info!("  GET    /api/search?q=query");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn create_document(
    State(database): State<Database>,
) -> AppResult<Json<CreateDocumentResponse>> {
    let id = database.create_document().await?;
    Ok(Json(CreateDocumentResponse { id }))
}

async fn get_document(
    Path(id): Path<String>,
    State(database): State<Database>,
) -> AppResult<Json<Document>> {
    let document = database.get_document(&id).await?;
    Ok(Json(document))
}

async fn update_document(
    Path(id): Path<String>,
    State(database): State<Database>,
    Json(payload): Json<UpdateDocumentRequest>,
) -> AppResult<Json<Document>> {
    // Validate input
    payload.validate().map_err(|e| {
        AppError::ValidationError(format!("Validation failed: {}", e))
    })?;

    // TODO: Extract real IP address from request
    let ip_address = "127.0.0.1";
    
    let document = database.update_document(&id, &payload.content, ip_address).await?;
    Ok(Json(document))
}

async fn get_document_history(
    Path(id): Path<String>,
    State(database): State<Database>,
) -> AppResult<Json<Vec<DocumentHistory>>> {
    let history = database.get_document_history(&id).await?;
    Ok(Json(history))
}

async fn get_document_stats(
    Path(id): Path<String>,
    State(database): State<Database>,
) -> AppResult<Json<serde_json::Value>> {
    let (history_count, last_updated) = database.get_document_stats(&id).await?;
    
    Ok(Json(serde_json::json!({
        "history_count": history_count,
        "last_updated": last_updated
    })))
}

async fn search_documents(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    State(database): State<Database>,
) -> AppResult<Json<Vec<Document>>> {
    let empty_string = String::new();
    let query = params.get("q").unwrap_or(&empty_string);
    
    if query.is_empty() {
        return Err(AppError::ValidationError("Search query 'q' is required".to_string()));
    }
    
    let documents = database.search_documents(query).await?;
    Ok(Json(documents))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;
    use axum::http::StatusCode;

    async fn create_test_app() -> TestServer {
        let database = Database::new("postgresql://collaborative_user:collaborative_password@localhost:5432/test_db").await.unwrap();
        
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
            .with_state(database);

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
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "Updated content");
    }
}
