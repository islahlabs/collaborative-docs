#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    use serde_json::json;
    use axum::http::StatusCode;

    use crate::{
        app::create_test_app,
        database::Database,
        models::{CreateDocumentResponse, Document, DocumentHistory},
    };

    async fn create_test_server() -> TestServer {
        let database = Database::new("postgresql://collaborative_user:collaborative_password@localhost:5432/test_db").await.unwrap();
        let app = create_test_app(database);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_create_document() {
        let server = create_test_server().await;
        
        let response = server
            .post("/api/doc")
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: CreateDocumentResponse = response.json();
        assert!(!body.id.is_empty());
    }

    #[tokio::test]
    async fn test_get_nonexistent_document() {
        let server = create_test_server().await;
        
        let response = server
            .get("/api/doc/nonexistent-id")
            .await;
        
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_and_get_document() {
        let server = create_test_server().await;
        
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
        let server = create_test_server().await;
        
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
        let server = create_test_server().await;
        
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