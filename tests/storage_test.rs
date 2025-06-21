use kode_ai_rs::document::Document;
use kode_ai_rs::storage::DocumentStorage;

#[test]
fn test_store_and_retrieve_document() {
    // Create a temporary directory for storage
    let temp_dir = tempfile::tempdir().unwrap();
    let mut storage = DocumentStorage::new(temp_dir.path()).unwrap();
    
    // Create a test document
    let doc = Document {
        path: "test.md".to_string(),
        content: "# Test Document\n\nThis is a test document.".to_string(),
        title: "Test Document".to_string(),
        summary: Some("This is a test document.".to_string()),
    };
    
    // Store the document
    storage.store_document(doc).unwrap();
    
    // Retrieve the document
    let retrieved = storage.get_document("test.md").unwrap();
    
    // Verify the document was stored correctly
    assert_eq!(retrieved.path, "test.md");
    assert_eq!(retrieved.title, "Test Document");
    assert_eq!(retrieved.summary, Some("This is a test document.".to_string()));
    assert_eq!(retrieved.content, "# Test Document\n\nThis is a test document.");
}

#[test]
fn test_get_all_documents() {
    // Create a temporary directory for storage
    let temp_dir = tempfile::tempdir().unwrap();
    let mut storage = DocumentStorage::new(temp_dir.path()).unwrap();
    
    // Create test documents
    let doc1 = Document {
        path: "doc1.md".to_string(),
        content: "# Document 1\n\nThis is document 1.".to_string(),
        title: "Document 1".to_string(),
        summary: Some("This is document 1.".to_string()),
    };
    
    let doc2 = Document {
        path: "doc2.md".to_string(),
        content: "# Document 2\n\nThis is document 2.".to_string(),
        title: "Document 2".to_string(),
        summary: Some("This is document 2.".to_string()),
    };
    
    // Store the documents
    storage.store_document(doc1).unwrap();
    storage.store_document(doc2).unwrap();
    
    // Get all documents
    let all_docs = storage.get_all_documents();
    
    // Verify we have both documents
    assert_eq!(all_docs.len(), 2);
    
    // Verify the documents are correct (order is not guaranteed)
    let paths: Vec<_> = all_docs.iter().map(|d| &d.path).collect();
    assert!(paths.contains(&&"doc1.md".to_string()));
    assert!(paths.contains(&&"doc2.md".to_string()));
}

#[test]
fn test_find_relevant_documents() {
    // Create a temporary directory for storage
    let temp_dir = tempfile::tempdir().unwrap();
    let mut storage = DocumentStorage::new(temp_dir.path()).unwrap();
    
    // Create test documents with different content
    let rust_doc = Document {
        path: "rust.md".to_string(),
        content: "# Rust Programming\n\nRust is a systems programming language focused on safety and performance.".to_string(),
        title: "Rust Programming".to_string(),
        summary: Some("Rust is a systems programming language focused on safety and performance.".to_string()),
    };
    
    let python_doc = Document {
        path: "python.md".to_string(),
        content: "# Python Programming\n\nPython is a high-level programming language known for its simplicity.".to_string(),
        title: "Python Programming".to_string(),
        summary: Some("Python is a high-level programming language known for its simplicity.".to_string()),
    };
    
    // Store the documents
    storage.store_document(rust_doc).unwrap();
    storage.store_document(python_doc).unwrap();
    
    // Search for Rust-related documents
    let rust_results = storage.find_relevant_documents("rust systems programming");
    
    // Verify we found the Rust document
    assert!(!rust_results.is_empty());
    assert_eq!(rust_results[0].path, "rust.md");
    
    // Search for Python-related documents
    let python_results = storage.find_relevant_documents("python high-level");
    
    // Verify we found the Python document
    assert!(!python_results.is_empty());
    assert_eq!(python_results[0].path, "python.md");
    
    // Search for a term that should match both documents
    let programming_results = storage.find_relevant_documents("programming language");
    
    // Verify we found both documents
    assert_eq!(programming_results.len(), 2);
    
    // Search for a term that shouldn't match any documents
    let no_results = storage.find_relevant_documents("javascript web development");
    
    // Verify we found no documents
    assert!(no_results.is_empty());
}