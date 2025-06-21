use anyhow::Result;
use pulldown_cmark::{html, Parser};
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a document with its content and metadata
#[derive(Debug, Clone)]
pub struct Document {
    pub path: String,
    pub content: String,
    pub title: String,
    pub summary: Option<String>,
}

/// Document scanner that finds and processes documentation files
pub struct DocumentScanner {
    supported_extensions: Vec<String>,
}

impl DocumentScanner {
    /// Create a new document scanner with default supported extensions
    pub fn new() -> Self {
        Self {
            supported_extensions: vec![
                "md".to_string(),
                "mdx".to_string(),
                "markdown".to_string(),
                "txt".to_string(),
                "rst".to_string(),
                "adoc".to_string(),
            ],
        }
    }

    /// Add a supported file extension
    pub fn add_extension(&mut self, extension: &str) {
        self.supported_extensions.push(extension.to_string());
    }

    /// Check if a file is a supported documentation file
    pub fn is_supported_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.supported_extensions.contains(&ext_str.to_lowercase());
            }
        }
        false
    }

    /// Scan a directory for documentation files
    pub fn scan_directory(&self, dir_path: &Path) -> Result<Vec<Document>> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && self.is_supported_file(path) {
                if let Ok(doc) = self.process_file(path) {
                    documents.push(doc);
                }
            }
        }

        Ok(documents)
    }

    /// Process a single documentation file
    pub fn process_file(&self, file_path: &Path) -> Result<Document> {
        let content = std::fs::read_to_string(file_path)?;
        let relative_path = self.get_relative_path(file_path)?;

        // Extract title from the content (first heading or filename)
        let title = self.extract_title(&content)
            .unwrap_or_else(|| {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string()
            });

        // Generate a summary if possible
        let summary = self.generate_summary(&content);

        Ok(Document {
            path: relative_path,
            content,
            title,
            summary,
        })
    }

    /// Extract the title from a markdown document (first heading)
    pub fn extract_title(&self, content: &str) -> Option<String> {
        let heading_regex = Regex::new(r"(?m)^#\s+(.+)$").ok()?;
        heading_regex.captures(content).map(|cap| cap[1].to_string())
    }

    /// Generate a summary from the document content
    pub fn generate_summary(&self, content: &str) -> Option<String> {
        // Take the first paragraph that's not a heading
        let paragraph_regex = Regex::new(r"(?m)^(?!#)(.+)$").ok()?;
        let mut summary = String::new();

        for cap in paragraph_regex.captures_iter(content) {
            let line = cap[1].trim();
            if !line.is_empty() {
                summary.push_str(line);
                summary.push(' ');

                // Limit summary length
                if summary.len() > 200 {
                    summary.truncate(197);
                    summary.push_str("...");
                    break;
                }
            }
        }

        if summary.is_empty() {
            None
        } else {
            Some(summary.trim().to_string())
        }
    }

    /// Convert markdown to plain text
    pub fn markdown_to_text(&self, markdown: &str) -> String {
        // Parse the markdown
        let parser = Parser::new(markdown);

        // Convert to HTML first
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Simple HTML to text conversion (very basic)
        let text = html_output
            .replace("<p>", "")
            .replace("</p>", "\n\n")
            .replace("<h1>", "")
            .replace("</h1>", "\n\n")
            .replace("<h2>", "")
            .replace("</h2>", "\n\n")
            .replace("<h3>", "")
            .replace("</h3>", "\n\n")
            .replace("<h4>", "")
            .replace("</h4>", "\n\n")
            .replace("<h5>", "")
            .replace("</h5>", "\n\n")
            .replace("<h6>", "")
            .replace("</h6>", "\n\n")
            .replace("<ul>", "")
            .replace("</ul>", "\n")
            .replace("<li>", "- ")
            .replace("</li>", "\n")
            .replace("<code>", "`")
            .replace("</code>", "`")
            .replace("<pre>", "```\n")
            .replace("</pre>", "\n```\n")
            .replace("<em>", "*")
            .replace("</em>", "*")
            .replace("<strong>", "**")
            .replace("</strong>", "**");

        text
    }

    /// Get the relative path of a file from the current directory
    fn get_relative_path(&self, path: &Path) -> Result<String> {
        let current_dir = std::env::current_dir()?;
        let path_buf = PathBuf::from(path);

        Ok(path_buf
            .strip_prefix(&current_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string())
    }
}

impl Default for DocumentScanner {
    fn default() -> Self {
        Self::new()
    }
}
