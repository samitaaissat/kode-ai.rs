use anyhow::Result;
use octocrab::Octocrab;
use std::path::Path;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use tempfile::TempDir;
use crate::document::Document;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicUsize, Ordering};

/// GitHub repository connector that handles authentication and repository operations
#[derive(Clone)]
pub struct GitHubConnector {
    client: Arc<Octocrab>,
    owner: String,
    pub repo: String,
    // Cache for file contents to avoid redundant API calls
    file_cache: Arc<RwLock<HashMap<String, String>>>,
    // Semaphore to limit concurrent requests to GitHub API
    request_semaphore: Arc<Semaphore>,
    // Counter for API requests to track rate limiting
    request_count: Arc<AtomicUsize>,
    // Maximum number of concurrent requests
    max_concurrent_requests: usize,
}

impl GitHubConnector {
    pub async fn new(owner: &str, repo: &str, token: Option<&str>) -> Result<Self> {
        // Validate parameters
        if owner.trim().is_empty() {
            anyhow::bail!("Owner cannot be empty");
        }
        if repo.trim().is_empty() {
            anyhow::bail!("Repository name cannot be empty");
        }

        let mut builder = Octocrab::builder();

        // Set personal access token if provided
        if let Some(token) = token {
            if token.trim().is_empty() {
                anyhow::bail!("Personal access token cannot be empty");
            }
            tracing::info!("Using personal access token for GitHub API");
            builder = builder.personal_token(token);
        }

        let client = builder.build()?;

        // Default to 5 concurrent requests to avoid rate limiting
        let max_concurrent_requests = 5;

        Ok(Self {
            client: Arc::new(client),
            owner: owner.to_string(),
            repo: repo.to_string(),
            file_cache: Arc::new(RwLock::new(HashMap::new())),
            request_semaphore: Arc::new(Semaphore::new(max_concurrent_requests)),
            request_count: Arc::new(AtomicUsize::new(0)),
            max_concurrent_requests,
        })
    }

    /// Get the contents of a file from the repository with retry logic
    pub async fn get_file_contents(&self, path: &str) -> Result<String> {
        // Check if the file is in the cache
        {
            let cache = self.file_cache.read().await;
            if let Some(content) = cache.get(path) {
                return Ok(content.clone());
            }
        }

        // Acquire a permit from the semaphore to limit concurrent requests
        let _permit = self.request_semaphore.clone().acquire_owned().await?;

        // Increment the request counter
        let request_number = self.request_count.fetch_add(1, Ordering::SeqCst);

        // Implement retry logic with exponential backoff
        let max_retries = 3;
        let mut retry_count = 0;
        let mut delay = Duration::from_millis(100);

        loop {
            // If we've exceeded the maximum number of retries, bail out
            if retry_count >= max_retries {
                anyhow::bail!("Failed to fetch file after {} retries", max_retries);
            }

            // If we're not on the first attempt, wait before retrying
            if retry_count > 0 {
                sleep(delay).await;
                // Exponential backoff: double the delay for each retry
                delay *= 2;
            }

            // Attempt to fetch the file
            match self.fetch_file_content(path).await {
                Ok(file_content) => {
                    // Store in cache
                    {
                        let mut cache = self.file_cache.write().await;
                        cache.insert(path.to_string(), file_content.clone());
                    }

                    return Ok(file_content);
                }
                Err(e) => {
                    // If we get a rate limit error or a network error, retry
                    if retry_count < max_retries {
                        retry_count += 1;
                        tracing::warn!("Retry {}/{} for file {}: {}", retry_count, max_retries, path, e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    /// Helper method to fetch file content from GitHub
    async fn fetch_file_content(&self, path: &str) -> Result<String> {
        let content = self
            .client
            .repos(&self.owner, &self.repo)
            .get_content()
            .path(path)
            .send()
            .await?;

        if let Some(file) = content.items.first() {
            if let Some(content) = &file.content {
                let decoded = STANDARD.decode(content.replace('\n', ""))?;
                let file_content = String::from_utf8(decoded)?;
                return Ok(file_content);
            }
        }

        anyhow::bail!("File not found or empty")
    }

    /// List all files in a directory recursively with parallel processing
    pub async fn list_files(&self, path: &str) -> Result<Vec<Document>> {
        // Use an iterative approach with a queue to avoid deep recursion
        let mut directories_to_process: Vec<String> = vec![path.to_string()];
        let scanner = crate::document::DocumentScanner::new();

        // First, collect all file paths to process
        let mut file_items = Vec::new();

        // Collect all files from all directories
        while let Some(current_path) = directories_to_process.pop() {
            let content = match self
                .client
                .repos(&self.owner, &self.repo)
                .get_content()
                .path(&current_path)
                .send()
                .await {
                    Ok(content) => content,
                    Err(e) => {
                        tracing::error!("Failed to list directory {}: {}", current_path, e);
                        continue;
                    }
                };

            for item in content.items {
                if item.r#type == "file" {
                    file_items.push((item.path, item.name));
                } else if item.r#type == "dir" {
                    // Add directory to the queue for processing
                    directories_to_process.push(item.path);
                }
            }
        }

        // Fetch file contents in parallel with controlled concurrency
        let mut file_contents = Vec::with_capacity(file_items.len());

        // Process files in chunks to control memory usage
        let chunk_size = self.max_concurrent_requests;
        for chunk in file_items.chunks(chunk_size) {
            let mut tasks = Vec::with_capacity(chunk.len());

            // Fetch each file's content in parallel
            for (item_path, _) in chunk {
                let item_path = item_path.clone();
                let self_clone = self.clone();

                // Spawn a task for each file to fetch its content
                let task = tokio::spawn(async move {
                    match self_clone.get_file_contents(&item_path).await {
                        Ok(content) => Some((item_path, content)),
                        Err(e) => {
                            tracing::error!("Failed to fetch file {}: {}", item_path, e);
                            None
                        }
                    }
                });

                tasks.push(task);
            }

            // Wait for all tasks in this chunk to complete
            for task in tasks {
                if let Ok(Some((path, content))) = task.await {
                    file_contents.push((path, content));
                }
            }
        }

        // Now process the file contents sequentially with a single scanner instance
        let mut documents = Vec::with_capacity(file_contents.len());

        for (path, content) in file_contents {
            // Extract filename from path for title fallback
            let filename = path.split('/').last().unwrap_or("Untitled").to_string();

            // Extract title from content or use filename
            let title = scanner.extract_title(&content)
                .unwrap_or_else(|| filename);

            // Generate a proper summary
            let summary = scanner.generate_summary(&content);

            let document = Document {
                path,
                content,
                title,
                summary,
            };

            documents.push(document);
        }

        Ok(documents)
    }
}
