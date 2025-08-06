use crate::{error::AppError, models::{Document, DocumentHistory}};
use sqlx::postgres::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::crdt::{DocumentManager, DocumentUpdate};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
    crdt_manager: Arc<RwLock<DocumentManager>>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = PgPool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Self { 
            pool,
            crdt_manager: Arc::new(RwLock::new(DocumentManager::new())),
        })
    }

    pub async fn create_document(&self) -> Result<String, AppError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        // Create in database
        sqlx::query!(
            "INSERT INTO documents (id, content, created_at, updated_at) VALUES ($1, $2, $3, $4)",
            id,
            "",
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        // Create in CRDT manager
        let mut manager = self.crdt_manager.write().await;
        manager.create_document(id.to_string());

        Ok(id.to_string())
    }

    pub async fn get_document(&self, id: &str) -> Result<Document, AppError> {
        let uuid = Uuid::parse_str(id).map_err(|_| AppError::DocumentNotFound(id.to_string()))?;
        
        // Try to get from CRDT first (for real-time updates)
        let crdt_manager = self.crdt_manager.read().await;
        
        if let Some(crdt_doc) = crdt_manager.get_document(id) {
            let content = crdt_doc.get_content();
            let now = chrono::Utc::now();
            
            return Ok(Document {
                id: id.to_string(),
                content,
                created_at: now, // We'd need to store this in CRDT too
                updated_at: now,
            });
        }
        
        // Fallback to database
        let row = sqlx::query!(
            "SELECT id, content, created_at, updated_at FROM documents WHERE id = $1",
            uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Document {
                id: row.id.to_string(),
                content: row.content,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }),
            None => Err(AppError::DocumentNotFound(id.to_string())),
        }
    }

    pub async fn update_document(&self, id: &str, content: &str, ip_address: &str) -> Result<Document, AppError> {
        let uuid = Uuid::parse_str(id).map_err(|_| AppError::DocumentNotFound(id.to_string()))?;
        let now = chrono::Utc::now();
        
        // Update in CRDT manager
        let mut manager = self.crdt_manager.write().await;
        let _update = manager.update_document(id, content, "user")
            .map_err(|e| AppError::InternalError(e))?;
        
        // Update in database (for persistence)
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE documents SET content = $1, updated_at = $2 WHERE id = $3",
            content,
            now,
            uuid
        )
        .execute(&mut *tx)
        .await?;

        // Add to history
        sqlx::query(
            "INSERT INTO document_history (document_id, content, ip_address, timestamp) VALUES ($1, $2, $3::inet, $4)"
        )
        .bind(uuid)
        .bind(content)
        .bind(ip_address)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Return the updated document directly instead of calling get_document
        // this circumvents call to get_document, which needs to acquire a read lock on the CRDT manager
        // and that can cause deadlocks when multiple updates are happening concurrently
        Ok(Document {
            id: id.to_string(),
            content: content.to_string(),
            created_at: now, // This should come from the database, but we'll use current time
            updated_at: now,
        })
    }

    pub async fn apply_crdt_update(&self, id: &str, update: &DocumentUpdate) -> Result<(), AppError> {
        let mut manager = self.crdt_manager.write().await;
        manager.apply_update(id, update)
            .map_err(|e| AppError::InternalError(e))?;
        Ok(())
    }

    pub async fn get_document_crdt_state(&self, id: &str) -> Result<crate::crdt::DocumentState, AppError> {
        let manager = self.crdt_manager.read().await;
        if let Some(doc) = manager.get_document(id) {
            Ok(doc.get_state())
        } else {
            Err(AppError::DocumentNotFound(id.to_string()))
        }
    }

    pub async fn get_document_history(&self, id: &str) -> Result<Vec<DocumentHistory>, AppError> {
        let uuid = Uuid::parse_str(id).map_err(|_| AppError::DocumentNotFound(id.to_string()))?;
        
        // First check if document exists
        let _document = self.get_document(id).await?;

        let rows = sqlx::query!(
            "SELECT content, ip_address::text, timestamp FROM document_history WHERE document_id = $1 ORDER BY timestamp ASC",
            uuid
        )
        .fetch_all(&self.pool)
        .await?;

        let history = rows
            .into_iter()
            .map(|row| DocumentHistory {
                content: row.content,
                ip_address: row.ip_address.unwrap_or_default(),
                timestamp: row.timestamp,
            })
            .collect();

        Ok(history)
    }

    // Additional PostgreSQL-specific methods for production features
    pub async fn get_document_stats(&self, id: &str) -> Result<(i64, chrono::DateTime<chrono::Utc>), AppError> {
        let uuid = Uuid::parse_str(id).map_err(|_| AppError::DocumentNotFound(id.to_string()))?;
        
        let row = sqlx::query!(
            "SELECT COUNT(*) as history_count, MAX(timestamp) as last_updated FROM document_history WHERE document_id = $1",
            uuid
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((row.history_count.unwrap_or(0), row.last_updated.unwrap_or_default()))
    }

    pub async fn search_documents(&self, query: &str) -> Result<Vec<Document>, AppError> {
        let rows = sqlx::query!(
            "SELECT id, content, created_at, updated_at FROM documents WHERE content ILIKE $1 ORDER BY updated_at DESC",
            format!("%{}%", query)
        )
        .fetch_all(&self.pool)
        .await?;

        let documents = rows
            .into_iter()
            .map(|row| Document {
                id: row.id.to_string(),
                content: row.content,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(documents)
    }
} 