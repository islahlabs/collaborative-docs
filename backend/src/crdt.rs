use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone)]
pub struct CRDTDocument {
    pub id: String,
    content: String,
    version: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DocumentUpdate {
    pub content: String,
    pub user_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DocumentState {
    pub content: String,
    pub version: u64,
    pub last_modified: i64,
}

impl CRDTDocument {
    pub fn new(id: String) -> Self {
        Self { 
            id, 
            content: String::new(),
            version: 0,
        }
    }

    pub fn from_existing(id: String, content: String) -> Self {
        Self { 
            id, 
            content,
            version: 0,
        }
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }

    pub fn update_content(&mut self, new_content: &str, user_id: &str) -> DocumentUpdate {
        self.content = new_content.to_string();
        self.version += 1;
        
        DocumentUpdate {
            content: new_content.to_string(),
            user_id: user_id.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn apply_update(&mut self, update: &DocumentUpdate) {
        self.content = update.content.clone();
        self.version += 1;
    }

    pub fn get_state(&self) -> DocumentState {
        DocumentState {
            content: self.content.clone(),
            version: self.version,
            last_modified: chrono::Utc::now().timestamp(),
        }
    }

    pub fn merge_update(&mut self, update: &DocumentUpdate) -> Result<(), String> {
        // Simple last-write-wins for now
        self.apply_update(update);
        Ok(())
    }

    pub fn get_diff(&self, _since_version: u64) -> Option<String> {
        // This would return the diff since the given version
        // For now, we'll return the full content
        Some(self.get_content())
    }
}

// Document manager to handle multiple documents
#[derive(Debug)]
pub struct DocumentManager {
    documents: HashMap<String, CRDTDocument>,
}

impl DocumentManager {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn create_document(&mut self, id: String) -> &CRDTDocument {
        let doc = CRDTDocument::new(id.clone());
        self.documents.insert(id.clone(), doc);
        self.documents.get(&id).unwrap()
    }

    pub fn get_document(&self, id: &str) -> Option<&CRDTDocument> {
        self.documents.get(id)
    }

    pub fn get_document_mut(&mut self, id: &str) -> Option<&mut CRDTDocument> {
        self.documents.get_mut(id)
    }

    pub fn update_document(&mut self, id: &str, content: &str, user_id: &str) -> Result<DocumentUpdate, String> {
        if let Some(doc) = self.documents.get_mut(id) {
            Ok(doc.update_content(content, user_id))
        } else {
            Err("Document not found".to_string())
        }
    }

    pub fn apply_update(&mut self, id: &str, update: &DocumentUpdate) -> Result<(), String> {
        if let Some(doc) = self.documents.get_mut(id) {
            doc.apply_update(update);
            Ok(())
        } else {
            Err("Document not found".to_string())
        }
    }
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
} 