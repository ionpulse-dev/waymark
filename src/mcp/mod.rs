//! The MCP server over stdio. All `rmcp` code is in this file, so a change of the library stays local.

pub mod approval;
pub mod approval_text;
pub mod args;
pub mod binding;
pub mod context;
pub mod error;
pub mod repo_url;
pub mod result;
pub mod schema;
pub mod tool;

use crate::config::Kind;
use context::Context;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool as McpTool, ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler};
use serde_json::Value;
use std::sync::Arc;
use tool::Registry;

pub const INSTRUCTIONS: &str = "waymark is a knowledge base of Markdown documents in Git. Every result has a message and a settings field with the limits that apply. A notices field lists things to pass on to the person.";

/// The server handler. It is cheap to clone.
#[derive(Clone)]
pub struct Server {
    ctx: Arc<Context>,
    registry: Arc<Registry>,
    /// The tool names that the client saw last. A change sends `tools/list_changed`.
    seen: Arc<std::sync::Mutex<Option<Vec<String>>>>,
}

impl Server {
    pub fn new(ctx: Context, registry: Registry) -> Server {
        Server {
            ctx: Arc::new(ctx),
            registry: Arc::new(registry),
            seen: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn names(&self) -> Vec<String> {
        self.registry
            .list(&self.ctx)
            .iter()
            .map(|t| t.name().to_string())
            .collect()
    }

    /// Remember the visible tools. Returns true if they differ from what the client saw before.
    fn changed(&self, now: Vec<String>) -> bool {
        let mut seen = self.seen.lock().unwrap();
        let changed = seen.as_ref().is_some_and(|old| *old != now);
        *seen = Some(now);
        changed
    }

    /// The tool list as MCP tools. Blocking: it reads files.
    pub fn list(&self) -> Vec<McpTool> {
        self.registry
            .list(&self.ctx)
            .into_iter()
            .map(|t| {
                let schema = match t.input_schema() {
                    Value::Object(m) => m,
                    _ => Default::default(),
                };
                let mut a = ToolAnnotations::new();
                a.read_only_hint = Some(t.kind() == Kind::Read);
                if t.kind() != Kind::Read {
                    a.destructive_hint = Some(t.kind() == Kind::Destructive);
                }
                McpTool::new(t.name(), t.description(), Arc::new(schema)).with_annotations(a)
            })
            .collect()
    }
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .build(),
        )
        .with_instructions(INSTRUCTIONS)
        .with_server_info(rmcp::model::Implementation::new(
            "waymark",
            env!("CARGO_PKG_VERSION"),
        ))
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let me = self.clone();
        let tools = tokio::task::spawn_blocking(move || {
            let tools = me.list();
            me.changed(tools.iter().map(|t| t.name.to_string()).collect());
            tools
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let me = self.clone();
        let name = request.name.to_string();
        let args = request.arguments.map(Value::Object).unwrap_or(Value::Null);
        // The channel to the client, for an approval question during this call.
        *self.ctx.channel.lock().unwrap() = Some(approval::Channel {
            peer: context.peer.clone(),
            handle: tokio::runtime::Handle::current(),
            progress_token: context.meta.get_progress_token(),
        });
        let (outcome, changed) = tokio::task::spawn_blocking(move || {
            let outcome = me.registry.call(&me.ctx, &name, args);
            // The limits are read at every call, so the list may have changed.
            let changed = me.changed(me.names());
            (outcome, changed)
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        if changed {
            let _ = context.peer.notify_tool_list_changed().await;
        }
        match outcome {
            Ok(o) => {
                let r = if o.is_error {
                    CallToolResult::structured_error(o.json)
                } else {
                    CallToolResult::structured(o.json)
                };
                Ok(r.into())
            }
            Err(tool::UnknownTool(n)) => {
                Err(McpError::invalid_params(format!("Unknown tool: {n}"), None))
            }
        }
    }
}

/// Serve MCP over stdio until the client closes the connection.
pub async fn serve_stdio(server: Server) -> Result<(), Box<dyn std::error::Error>> {
    use rmcp::ServiceExt;
    let running = server.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;
    Ok(())
}
