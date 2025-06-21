use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::document::Document;

/// Document storage that handles storing and retrieving documents
pub struct DocumentStorage {
    storage_path: PathBuf,
    documents: HashMap<String, StoredDocument>,
}

/// Stored document with additional metadata for retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDocument {
    pub path: String,
    pub content: String,
    pub title: String,
    pub summary: Option<String>,
    pub keywords: Vec<String>,
}

impl DocumentStorage {
    /// Create a new document storage with the given storage path
    pub fn new(storage_path: impl AsRef<Path>) -> Result<Self> {
        let storage_path = storage_path.as_ref().to_path_buf();
        
        // Create the storage directory if it doesn't exist
        if !storage_path.exists() {
            fs::create_dir_all(&storage_path)?;
        }
        
        // Try to load existing documents
        let documents = Self::load_documents(&storage_path).unwrap_or_default();
        
        Ok(Self {
            storage_path,
            documents,
        })
    }
    
    /// Store a document
    pub fn store_document(&mut self, document: Document) -> Result<()> {
        // Extract keywords from the document content
        let keywords = self.extract_keywords(&document.content);
        
        // Create a stored document
        let stored_document = StoredDocument {
            path: document.path.clone(),
            content: document.content,
            title: document.title,
            summary: document.summary,
            keywords,
        };
        
        // Add to the in-memory storage
        self.documents.insert(document.path, stored_document);
        
        // Save to disk
        self.save_documents()?;
        
        Ok(())
    }
    
    /// Store multiple documents
    pub fn store_documents(&mut self, documents: Vec<Document>) -> Result<()> {
        for document in documents {
            // Extract keywords from the document content
            let keywords = self.extract_keywords(&document.content);
            
            // Create a stored document
            let stored_document = StoredDocument {
                path: document.path.clone(),
                content: document.content,
                title: document.title,
                summary: document.summary,
                keywords,
            };
            
            // Add to the in-memory storage
            self.documents.insert(document.path, stored_document);
        }
        
        // Save to disk
        self.save_documents()?;
        
        Ok(())
    }
    
    /// Get all stored documents
    pub fn get_all_documents(&self) -> Vec<&StoredDocument> {
        self.documents.values().collect()
    }
    
    /// Get a specific document by path
    pub fn get_document(&self, path: &str) -> Option<&StoredDocument> {
        self.documents.get(path)
    }
    
    /// Find documents relevant to a query
    pub fn find_relevant_documents(&self, query: &str) -> Vec<&StoredDocument> {
        let query_keywords = self.extract_keywords(query);
        
        // Score documents based on keyword matches
        let mut scored_documents: Vec<(&StoredDocument, usize)> = self
            .documents
            .values()
            .map(|doc| {
                let score = query_keywords
                    .iter()
                    .filter(|kw| doc.keywords.contains(kw))
                    .count();
                (doc, score)
            })
            .filter(|(_, score)| *score > 0)
            .collect();
        
        // Sort by score (descending)
        scored_documents.sort_by(|(_, score1), (_, score2)| score2.cmp(score1));
        
        // Return the documents
        scored_documents.into_iter().map(|(doc, _)| doc).collect()
    }
    
    /// Extract keywords from text
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        let text = text.to_lowercase();
        
        // Split by non-alphanumeric characters
        let words: Vec<&str> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();
        
        // Filter out common words and short words
        let stopwords = [
            "the", "a", "an", "and", "or", "but", "if", "then", "else", "when",
            "at", "from", "by", "for", "with", "about", "against", "between",
            "into", "through", "during", "before", "after", "above", "below",
            "to", "of", "in", "on", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would",
            "shall", "should", "can", "could", "may", "might", "must", "this",
            "that", "these", "those", "i", "you", "he", "she", "it", "we", "they",
        ];
        
        let keywords: Vec<String> = words
            .into_iter()
            .filter(|word| word.len() > 2 && !stopwords.contains(word))
            .map(|s| s.to_string())
            .collect();
        
        // Deduplicate
        let mut unique_keywords = Vec::new();
        for keyword in keywords {
            if !unique_keywords.contains(&keyword) {
                unique_keywords.push(keyword);
            }
        }
        
        unique_keywords
    }
    
    /// Save documents to disk
    fn save_documents(&self) -> Result<()> {
        let index_path = self.storage_path.join("documents.json");
        let file = File::create(index_path)?;
        let writer = BufWriter::new(file);
        
        serde_json::to_writer(writer, &self.documents)?;
        
        Ok(())
    }
    
    /// Load documents from disk
    fn load_documents(storage_path: &Path) -> Result<HashMap<String, StoredDocument>> {
        let index_path = storage_path.join("documents.json");
        
        if !index_path.exists() {
            return Ok(HashMap::new());
        }
        
        let file = File::open(index_path)?;
        let reader = BufReader::new(file);
        
        let documents: HashMap<String, StoredDocument> = serde_json::from_reader(reader)?;
        
        Ok(documents)
    }
}