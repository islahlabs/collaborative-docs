use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentHistory {
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateDocumentRequest {
    #[validate(length(min = 0, max = 100000, message = "Content must be between 0 and 100,000 characters"))]
    pub content: String,
}

impl Document {
    pub fn new(id: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}

impl DocumentHistory {
    pub fn new(content: String, ip_address: String) -> Self {
        Self {
            timestamp: Utc::now(),
            ip_address,
            content,
        }
    }
} 