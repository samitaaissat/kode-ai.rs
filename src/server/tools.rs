use std::sync::Arc;
use rmcp::model::{AnnotateAble, CallToolResult, Content, Implementation, ListResourcesResult, PaginatedRequestParam, ProtocolVersion, RawResource, Resource, ServerCapabilities, ServerInfo};
use tokio::sync::{RwLock};
use serde_json::{json};
use serde::Deserialize;

use rmcp::{
    Error as McpError, RoleServer, ServerHandler, model::*, schemars,
    service::RequestContext, tool,
};

use crate::storage::DocumentStorage;


type DocumentStore = Arc<RwLock<DocumentStorage>>;

#[derive(Clone)]
pub struct Documents{
    pub store: DocumentStore,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAllDocsRequest {
    #[schemars(description = "the maximum number of documents to return", default)]
    pub limit: i32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetDocumentRequest {
    #[schemars(description = "the path of the document to retrieve")]
    path: String
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindRelevantDocsRequest {
    #[schemars(description = "the query to search for relevant documents")]
    pub query: String,

    #[schemars(description = "the maximum number of documents to return", default)]
    pub limit: i32,
}

#[tool(tool_box)]
impl Documents {

    pub fn new(store: DocumentStore) -> Self {
        Self {
            store,
        }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    #[tool(description = "Get all documents in the storage")]
    async fn get_all_docs(&self, #[tool(aggr)] GetAllDocsRequest {limit} : GetAllDocsRequest) -> Result<CallToolResult, McpError> {
        let store = self.store.read().await;
        let docs = store.get_all_documents();

        let records: Vec<_> = docs
            .iter()
            .map(|doc| {
                json!({
                    "path": doc.path,
                    "title": doc.title,
                    "summary": doc.summary,
                })
            })
            .collect();

        // Limit the number of documents returned
        let records: Vec<_> = if limit > 0 && limit < records.len() as i32 {
            records.into_iter().take(limit as usize).collect()
        } else {
            records
        };

        let response = json!({
            "total": docs.len(),
            "returned": records.len(),
            "documents": records
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    #[tool(description = "Get a specific document by path")]
    async fn get_document(
        &self,
        #[tool(aggr)] GetDocumentRequest { path }: GetDocumentRequest,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.read().await;
        if let Some(doc) = store.get_document(&path) {
            let response = json!({
                "path": doc.path,
                "title": doc.title,
                "summary": doc.summary,
                "content": doc.content,
            });
            Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
        } else {
            Err(McpError::resource_not_found("document_not_found", Some(json!({ "path": path }))))
        }
    }

    #[tool(description = "Find documents relevant to a query")]
    async fn find_relevant_docs(
        &self,
        #[tool(aggr)] FindRelevantDocsRequest { query, limit }: FindRelevantDocsRequest,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.read().await;
        let docs = store.find_relevant_documents(&query);

        if docs.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "documents": [],
                    "message": "No relevant documents found for the query"
                }).to_string(),
            )]));
        }

        let records: Vec<_> = docs
            .iter()
            .map(|doc| {
                json!({
                    "path": doc.path,
                    "title": doc.title,
                    "summary": doc.summary,
                    "content": doc.content,
                })
            })
            .collect();

        // Limit the number of documents returned
        let records: Vec<_> = if limit > 0 && limit < records.len() as i32 {
            records.into_iter().take(limit as usize).collect()
        } else {
            records
        };

        let response = json!({
            "total": docs.len(),
            "returned": records.len(),
            "documents": records
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }
}


#[tool(tool_box)]
impl ServerHandler for Documents {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides tools to access documentation from a GitHub repository. Use 'get_all_docs' to retrieve all available documents, 'get_document' to fetch a specific document by path, or 'find_relevant_docs' to search for documents relevant to a query.".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                self._create_resource_text("str:////Users/to/some/path/", "cwd"),
                self._create_resource_text("memo://insights", "memo-name"),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "str:////Users/to/some/path/" => {
                let cwd = "/Users/to/some/path/";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(cwd, uri)],
                })
            }
            "memo://insights" => {
                let memo = "Business Intelligence Memo\n\nAnalysis has revealed 5 key insights ...";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(memo, uri)],
                })
            }
            _ => Err(McpError::resource_not_found(
                "resource_not_found",
                Some(json!({
                    "uri": uri
                })),
            )),
        }
    }

    async fn list_prompts(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![Prompt::new(
                "example_prompt",
                Some("This is an example prompt that takes one required agrument, message"),
                Some(vec![PromptArgument {
                    name: "message".to_string(),
                    description: Some("A message to put in the prompt".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }

    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, arguments: _ }: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match name.as_str() {
            "example_prompt" => {
                let prompt = "This is an example prompt with your message here: '{message}'";
                Ok(GetPromptResult {
                    description: None,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt),
                    }],
                })
            }
            _ => Err(McpError::invalid_params("prompt not found", None)),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }
}
