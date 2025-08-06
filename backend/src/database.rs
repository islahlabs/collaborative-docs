use crate::{error::AppError, models::{Document, DocumentHistory}};
use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = PgPool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Self { pool })
    }

    pub async fn create_document(&self) -> Result<String, AppError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        sqlx::query!(
            "INSERT INTO documents (id, content, created_at, updated_at) VALUES ($1, $2, $3, $4)",
            id,
            "",
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(id.to_string())
    }

    pub async fn get_document(&self, id: &str) -> Result<Document, AppError> {
        let uuid = Uuid::parse_str(id).map_err(|_| AppError::DocumentNotFound(id.to_string()))?;
        
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
        
        // Start a transaction
        let mut tx = self.pool.begin().await?;

        // Update the document
        sqlx::query!(
            "UPDATE documents SET content = $1, updated_at = $2 WHERE id = $3",
            content,
            now,
            uuid
        )
        .execute(&mut *tx)
        .await?;

        // Add to history - use raw SQL to avoid type issues
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

        // Return the updated document
        self.get_document(id).await
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