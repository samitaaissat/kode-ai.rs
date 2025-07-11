use std::sync::Arc;
use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tokio::sync::RwLock;
use kode_ai_rs::server::Documents;
use kode_ai_rs::storage::DocumentStorage;
use clap::Parser;
use kode_ai_rs::github::GitHubConnector;

#[cfg(feature = "trace")]
use tracing_subscriber::{EnvFilter};

#[derive(Parser)]
struct Cli {
    /// A github repository to scan for documentation
    #[clap(long, default_value = "rust-sdk")]
    github_repo: String,
    /// A github repository subfolder to scan for documentation (optional)
    #[clap(long, default_value = "")]
    github_subfolder: String,
    /// A github repository owner (optional)
    #[clap(long, default_value = "modelcontextprotocol")]
    github_owner: String,
    /// A github personal access token to use for authentication (optional)
    #[clap(long)]
    github_pat: Option<String>,
}

/// You can inspect the server using the Model Context Protocol Inspector.
/// npx @modelcontextprotocol/inspector cargo run -p kode-ai-rs

#[tokio::main]
async fn main() -> Result<()> {
    // Get command line arguments
    let args = Cli::parse();

    // Initialize the tracing subscriber with file and stdout logging
    #[cfg(feature = "trace")]
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::fs::File::create("server.log")?)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP server");

    // Document storage initialization in temporary directory
    let temp_dir = tempfile::tempdir()?;
    tracing::info!("Using temporary directory for document storage: {:?}", temp_dir.path());
    let mut store = DocumentStorage::new(temp_dir.path())?;
    tracing::info!("Document storage initialized at: {:?}", temp_dir.path());

    // Setup Github connector
    let github_connector = if !args.github_repo.is_empty() {
        Some(GitHubConnector::new(
            &args.github_owner,
            &args.github_repo,
            args.github_pat.as_deref(),
        ).await?)
    } else {
        tracing::info!("No github repository specified, skipping");
        None
    };

    if let Some(connector) = &github_connector {
        tracing::info!("Scanning GitHub repository {} in subfolder: {}", connector.repo, args.github_subfolder);
        match connector.list_files(&args.github_subfolder).await {
            Ok(documents) => {
                tracing::info!("Found {} documents in the repository", documents.len());
                store.store_documents(documents)?;
            }
            Err(e) => {
                tracing::error!("Failed to scan GitHub repository: {}", e);
            }
        }
    }

    let service = Documents::new(Arc::new(RwLock::new(store)))
        .serve(stdio()).await.inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}