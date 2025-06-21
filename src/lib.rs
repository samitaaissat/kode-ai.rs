//! # Kode AI MCP Server
//!
//! A Model-Context-Provider (MCP) server that interfaces with a GitHub repository
//! to scan for documentation files and provide them to an LLM for contextualization.
//!
//! ## Features
//!
//! - Scan GitHub repositories for documentation files (md, mdx, etc.)
//! - Store documents in a format suitable for LLM contextualization
//! - Provide tools to get all documentation or find relevant documentation based on a query
//!
//! ## Modules
//!
//! - `server`: MCP server implementation and tools
//! - `storage`: Document storage and retrieval
//! - `document`: Document processing and parsing
//! - `github`: GitHub API integration for fetching documents

/// Server implementation and MCP tools
pub mod server;
/// Document storage and retrieval
pub mod storage;
/// Document processing and parsing
pub mod document;
/// GitHub API integration
pub mod github;
