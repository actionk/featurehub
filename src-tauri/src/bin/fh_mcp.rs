use std::sync::Mutex;

use clap::Parser;
use feature_hub::{config, db, extensions};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_router,
    transport::stdio,
    ServerHandler, ServiceExt,
};
use rusqlite::Connection;

// ─── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "fh-mcp", about = "FeatureHub MCP server for Claude Code")]
struct Cli {
    /// Scope tools to a specific feature ID
    #[arg(long)]
    feature: Option<String>,

    /// The Claude Code session ID (passed by fh CLI for context injection)
    #[arg(long)]
    session_id: Option<String>,
}

// ─── Parameter structs ───────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ListFeaturesParams {
    /// Filter by status (e.g. "todo", "in_progress", "in_review", "done")
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct FeatureIdParam {
    /// The feature UUID
    feature_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SearchParam {
    /// Search query
    query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateTaskParams {
    /// The feature UUID to add the task to
    feature_id: String,
    /// Task title
    title: String,
    /// Task source: "manual" (default) or "jira"
    #[serde(default)]
    source: Option<String>,
    /// External ticket key (e.g. "PROJ-123")
    #[serde(default)]
    external_key: Option<String>,
    /// Full external URL (e.g. Jira ticket URL)
    #[serde(default)]
    external_url: Option<String>,
    /// External status string (e.g. "In Progress")
    #[serde(default)]
    external_status: Option<String>,
    /// Ticket summary/description
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UpdateTaskParams {
    /// The task UUID
    id: String,
    /// New title (optional)
    #[serde(default)]
    title: Option<String>,
    /// Mark done or not done (optional)
    #[serde(default)]
    done: Option<bool>,
    /// Updated external status (optional)
    #[serde(default)]
    external_status: Option<String>,
    /// Updated description (optional)
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DeleteTaskParams {
    /// The task UUID
    id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SubmitPlanParams {
    /// The feature UUID to submit the plan for
    feature_id: String,
    /// Plan title (short summary)
    title: String,
    /// Full plan body in Markdown
    body: String,
    /// Optional Claude session ID for traceability
    #[serde(default)]
    session_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GetPlanStatusParams {
    /// The plan UUID to check
    plan_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UpdatePlanParams {
    /// The plan UUID to update
    plan_id: String,
    /// New title (optional)
    #[serde(default)]
    title: Option<String>,
    /// New body in Markdown (optional)
    #[serde(default)]
    body: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct MoveSessionParams {
    /// The session UUID to move
    session_id: String,
    /// The target feature UUID to move the session to
    target_feature_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct LinkSessionParams {
    /// The feature UUID to link this session to
    feature_id: String,
    /// The Claude Code session ID (from your current session)
    claude_session_id: String,
    /// Optional title for the session
    #[serde(default)]
    title: Option<String>,
    /// Optional project path
    #[serde(default)]
    project_path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SaveNoteParams {
    /// The feature UUID
    feature_id: String,
    /// Markdown note content
    content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SaveContextParams {
    /// The feature UUID
    feature_id: String,
    /// Persistent context/instructions content
    content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateFeatureParams {
    /// Feature title
    title: String,
    /// Ticket ID / external reference (optional, e.g. "HUB-123")
    #[serde(default)]
    ticket_id: Option<String>,
    /// Initial status (optional, defaults to "todo"): "todo", "in_progress", "in_review", "done"
    #[serde(default)]
    status: Option<String>,
    /// Feature description / project overview (optional)
    #[serde(default)]
    description: Option<String>,
    /// Parent feature ID to nest this under (optional, omit for root-level)
    #[serde(default)]
    parent_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SetFeatureParentParams {
    /// The feature UUID
    feature_id: String,
    /// Parent feature ID to nest under, or empty string / null to make root-level
    #[serde(default)]
    parent_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DeleteFeatureParams {
    /// The feature UUID to delete
    feature_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TogglePinParams {
    /// The feature UUID
    feature_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SetArchivedParams {
    /// The feature UUID
    feature_id: String,
    /// Whether to archive (true) or unarchive (false)
    archived: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UpdateFeatureParams {
    /// The feature UUID
    id: String,
    /// New title (optional)
    #[serde(default)]
    title: Option<String>,
    /// New description / project overview (optional)
    #[serde(default)]
    description: Option<String>,
    /// New status (optional): "todo", "in_progress", "in_review", "done"
    #[serde(default)]
    status: Option<String>,
    /// Parent feature ID to nest this feature under (optional, null to make root-level)
    #[serde(default)]
    parent_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct AddLinkParams {
    /// The feature UUID
    feature_id: String,
    /// Link title
    title: String,
    /// URL
    url: String,
    /// Link type (optional, auto-detected from URL if omitted)
    #[serde(default)]
    link_type: Option<String>,
    /// Optional description/note for this link (e.g. "Epic ticket", "Story ticket", "Design doc")
    #[serde(default)]
    description: Option<String>,
    /// Optional JSON metadata for the link. For Jira links, include: {"key": "HUB-123", "status": "To Do", "issue_type": "Story", "summary": "Short description", "assignee": "Name"}
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DeleteLinkParams {
    /// The link UUID to delete
    id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UpdateLinkParams {
    /// The link UUID to update
    id: String,
    /// New title (optional, keeps existing if omitted)
    #[serde(default)]
    title: Option<String>,
    /// New URL (optional, keeps existing if omitted)
    #[serde(default)]
    url: Option<String>,
    /// New link type (optional, auto-detected from URL if omitted)
    #[serde(default)]
    link_type: Option<String>,
    /// New description (optional, use null to clear, omit to keep existing)
    #[serde(default)]
    description: Option<Option<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateTagParams {
    /// Tag name
    name: String,
    /// Tag color as hex string (e.g. "#7c5bf5"). Random color used if omitted.
    #[serde(default)]
    color: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ToggleTagParams {
    /// The feature UUID
    feature_id: String,
    /// The tag UUID
    tag_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CloneRepositoryParams {
    /// The feature UUID
    feature_id: String,
    /// Git repository URL to clone
    repo_url: String,
    /// Optional name for the cloned directory (derived from URL if not provided)
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SaveSettingsParams {
    /// Path to the fh CLI binary (optional, null to clear)
    #[serde(default)]
    fh_cli_path: Option<String>,
    /// MCP servers to configure. Each object has: name, command, args (array), env (object of key-value pairs)
    #[serde(default)]
    mcp_servers: Option<Vec<McpServerParam>>,
    /// Default repositories to configure. Each object has: url (string), name (optional), description (optional)
    #[serde(default)]
    default_repositories: Option<Vec<RepositoryParam>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RepositoryParam {
    /// Git repository URL
    url: String,
    /// Optional display name (derived from URL if not provided)
    #[serde(default)]
    name: Option<String>,
    /// Optional description (shown to Claude during feature initialization)
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct McpServerParam {
    /// Server name (used as key in .mcp.json)
    name: String,
    /// Command to run (e.g. "npx", "node", "python")
    command: String,
    /// Arguments for the command
    args: Vec<String>,
    /// Environment variables (key-value pairs)
    #[serde(default)]
    env: std::collections::HashMap<String, String>,
    /// Whether this server is enabled by default for all features
    default_enabled: Option<bool>,
    /// URL for streamable HTTP MCP servers (e.g. hosted MCP endpoints)
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateSkillParams {
    /// A unique identifier for the skill (lowercase, kebab-case, e.g. "code-review")
    id: String,
    /// Human-readable skill name (e.g. "Code Review Guidelines")
    name: String,
    /// The skill content/instructions in Markdown. This is injected into Claude sessions when the skill is enabled for a feature.
    content: String,
    /// Whether this skill is enabled by default for all features (defaults to true)
    #[serde(default)]
    default_enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UpdateSkillParams {
    /// The skill ID to update
    id: String,
    /// New name (optional)
    #[serde(default)]
    name: Option<String>,
    /// New content/instructions in Markdown (optional)
    #[serde(default)]
    content: Option<String>,
    /// Whether this skill is enabled by default (optional)
    #[serde(default)]
    default_enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DeleteSkillParams {
    /// The skill ID to delete
    id: String,
}

// ─── Knowledge Base params ──────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ListKnowledgeParams {
    /// Optional folder UUID to filter entries
    #[serde(default)]
    folder_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct KnowledgeEntryIdParam {
    /// The knowledge entry UUID
    id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateKnowledgeEntryParams {
    /// Entry title
    title: String,
    /// Full markdown content
    content: String,
    /// One-line summary (shown in TOC)
    #[serde(default)]
    description: Option<String>,
    /// Folder UUID to place entry in
    #[serde(default)]
    folder_id: Option<String>,
    /// Feature UUID to link entry to
    #[serde(default)]
    feature_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UpdateKnowledgeEntryParams {
    /// The knowledge entry UUID
    id: String,
    /// New title (optional)
    #[serde(default)]
    title: Option<String>,
    /// New content (optional)
    #[serde(default)]
    content: Option<String>,
    /// New description (optional)
    #[serde(default)]
    description: Option<String>,
    /// Move to folder (optional, null for root)
    #[serde(default)]
    folder_id: Option<Option<String>>,
    /// Link to feature (optional, null to unlink)
    #[serde(default)]
    feature_id: Option<Option<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateKnowledgeFolderParams {
    /// Folder name
    name: String,
    /// Parent folder UUID (null for root)
    #[serde(default)]
    parent_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RenameKnowledgeFolderParams {
    /// The folder UUID
    id: String,
    /// New folder name
    name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DeleteKnowledgeFolderParams {
    /// The folder UUID
    id: String,
}

// ─── MCP Server ──────────────────────────────────────────────────────────────

#[derive(Clone)]
struct FeatureHubMcp {
    db: std::sync::Arc<Mutex<Connection>>,
    default_feature_id: Option<String>,
    /// The Claude session ID passed via CLI (auto-populated in submit_plan if not provided).
    claude_session_id: Option<String>,
    /// Path to the active storage directory (for loading per-storage settings).
    storage_path: std::path::PathBuf,
    /// Pre-loaded context about the scoped feature, embedded in server instructions.
    feature_context: Option<String>,
    tool_router: ToolRouter<Self>,
    extension_registry: extensions::ExtensionRegistry,
}

/// Helper to run a blocking DB closure on tokio's blocking pool.
macro_rules! with_db {
    ($self:expr, $conn:ident => $body:expr) => {{
        let db = $self.db.clone();
        tokio::task::spawn_blocking(move || {
            let $conn = db.lock().map_err(|e| {
                rmcp::ErrorData::internal_error(format!("DB lock error: {}", e), None)
            })?;
            $body
        })
        .await
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Task join error: {}", e), None))?
    }};
}

/// Convert any serializable value to a JSON text content result.
fn json_result<T: serde::Serialize>(val: &T) -> Result<CallToolResult, rmcp::ErrorData> {
    let text = serde_json::to_string_pretty(val)
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialize error: {}", e), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

fn db_err(e: String) -> rmcp::ErrorData {
    rmcp::ErrorData::internal_error(e, None)
}

#[tool_router]
impl FeatureHubMcp {
    fn new(
        conn: Connection,
        default_feature_id: Option<String>,
        claude_session_id: Option<String>,
        storage_path: std::path::PathBuf,
    ) -> Self {
        let feature_context = default_feature_id.as_ref().and_then(|id| {
            build_feature_context(&conn, id, claude_session_id.as_deref(), &storage_path).ok()
        });
        let extension_registry =
            extensions::ExtensionRegistry::load_from_dir(&storage_path.join("extensions"));
        Self {
            db: std::sync::Arc::new(Mutex::new(conn)),
            default_feature_id,
            claude_session_id,
            storage_path,
            feature_context,
            tool_router: Self::tool_router(),
            extension_registry,
        }
    }

    // ── Read tools ───────────────────────────────────────────────────────

    #[tool(description = "List all features, optionally filtered by status")]
    async fn list_features(
        &self,
        Parameters(p): Parameters<ListFeaturesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let features = db::features::get_features(&conn, p.status, None).map_err(db_err)?;
            json_result(&features)
        })
    }

    #[tool(description = "Get full details for a feature including tags, links, and directories")]
    async fn get_feature(
        &self,
        Parameters(p): Parameters<FeatureIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let feature = db::features::get_feature(&conn, &p.feature_id).map_err(db_err)?;
            json_result(&feature)
        })
    }

    #[tool(description = "List tasks for a feature")]
    async fn get_tasks(
        &self,
        Parameters(p): Parameters<FeatureIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let tasks = db::tasks::get_tasks(&conn, &p.feature_id).map_err(db_err)?;
            json_result(&tasks)
        })
    }

    #[tool(description = "Get links for a feature")]
    async fn get_links(
        &self,
        Parameters(p): Parameters<FeatureIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let links = db::links::get_links(&conn, &p.feature_id).map_err(db_err)?;
            json_result(&links)
        })
    }

    #[tool(description = "Get the note for a feature")]
    async fn get_note(
        &self,
        Parameters(p): Parameters<FeatureIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let note = db::notes::get_note(&conn, &p.feature_id).map_err(db_err)?;
            json_result(&note)
        })
    }

    #[tool(description = "List Claude Code sessions for a feature")]
    async fn get_sessions(
        &self,
        Parameters(p): Parameters<FeatureIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let sessions = db::sessions::get_sessions(&conn, &p.feature_id).map_err(db_err)?;
            json_result(&sessions)
        })
    }

    #[tool(description = "List attached files for a feature")]
    async fn get_files(
        &self,
        Parameters(p): Parameters<FeatureIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let sp = self.storage_path.clone();
        with_db!(self, conn => {
            let mut files = db::files::get_files(&conn, &p.feature_id).map_err(db_err)?;
            for f in &mut files {
                f.stored_path = feature_hub::paths::resolve_path_string(&f.stored_path, &sp);
            }
            json_result(&files)
        })
    }

    #[tool(
        description = "List cloned repositories for a feature. Each has clone_status (ready/cloning/failed), repo_url, and local path."
    )]
    async fn get_repositories(
        &self,
        Parameters(p): Parameters<FeatureIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let sp = self.storage_path.clone();
        with_db!(self, conn => {
            let mut dirs = db::directories::get_directories(&conn, &p.feature_id).map_err(db_err)?;
            for d in &mut dirs {
                d.path = feature_hub::paths::resolve_path_string(&d.path, &sp);
            }
            json_result(&dirs)
        })
    }

    #[tool(
        description = "List default repositories from app settings. These are predefined repo URLs that can be cloned for features."
    )]
    async fn get_default_repositories(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let storage_settings = config::load_storage_settings(&self.storage_path).map_err(db_err)?;
        json_result(&storage_settings.default_repositories)
    }

    #[tool(
        description = "Clone a repository into a feature's workspace. Performs a shallow clone (--depth 1). Use get_default_repositories to see available repos."
    )]
    async fn clone_repository(
        &self,
        Parameters(p): Parameters<CloneRepositoryParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        let repo_url = p.repo_url.clone();

        // Derive name from URL if not provided
        let repo_name = p.name.unwrap_or_else(|| {
            repo_url
                .rsplit('/')
                .next()
                .unwrap_or("repo")
                .trim_end_matches(".git")
                .to_string()
        });

        // Compute target path
        let target_path = self
            .storage_path
            .join("workspaces")
            .join(&fid)
            .join(&repo_name);

        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| db_err(e.to_string()))?;
        }

        // Store relative path in DB
        let target_str = feature_hub::paths::to_storage_relative(
            &target_path.to_string_lossy(),
            &self.storage_path,
        );

        // Insert directory record
        let dir = {
            let conn = self.db.lock().map_err(|e| db_err(e.to_string()))?;
            db::directories::add_directory(
                &conn,
                &fid,
                &target_str,
                Some(repo_name.clone()),
                Some(repo_url.clone()),
                Some("cloning".to_string()),
            )
            .map_err(db_err)?
        };

        // MCP is synchronous stdio, so we clone blocking
        match feature_hub::git::clone_repo(&repo_url, &target_path) {
            Ok(()) => {
                let conn = self.db.lock().map_err(|e| db_err(e.to_string()))?;
                db::directories::update_clone_status(&conn, &dir.id, "ready", None)
                    .map_err(db_err)?;
                let _ = config::push_notification(
                    &format!("Repository cloned: {}", repo_name),
                    Some(&fid),
                );
                let updated = db::directories::get_directory(&conn, &dir.id).map_err(db_err)?;
                json_result(&updated)
            }
            Err(e) => {
                let conn = self.db.lock().map_err(|e| db_err(e.to_string()))?;
                let _ = db::directories::update_clone_status(&conn, &dir.id, "failed", Some(&e));
                Err(rmcp::ErrorData::internal_error(
                    format!("Clone failed: {}", e),
                    None,
                ))
            }
        }
    }

    #[tool(description = "List all tags")]
    async fn get_tags(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let tags = db::tags::get_tags(&conn).map_err(db_err)?;
            json_result(&tags)
        })
    }

    #[tool(description = "Create a new tag. Returns the created tag with its ID.")]
    async fn create_tag(
        &self,
        Parameters(p): Parameters<CreateTagParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let name = p.name.clone();
        let color = p.color.unwrap_or_else(|| "#7c5bf5".to_string());
        with_db!(self, conn => {
            let tag = db::tags::create_tag(&conn, &name, &color).map_err(db_err)?;
            let _ = config::push_notification(&format!("Tag created: {}", tag.name), None);
            json_result(&tag)
        })
    }

    #[tool(
        description = "Toggle a tag on a feature. If the tag is not assigned, it will be added; if already assigned, it will be removed."
    )]
    async fn toggle_tag(
        &self,
        Parameters(p): Parameters<ToggleTagParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        let tid = p.tag_id.clone();
        with_db!(self, conn => {
            db::tags::toggle_tag(&conn, &fid, &tid).map_err(db_err)?;
            let _ = config::push_notification("Tag toggled", Some(&fid));
            Ok(CallToolResult::success(vec![Content::text("Tag toggled successfully")]))
        })
    }

    #[tool(description = "Full-text search across all features, links, sessions, and notes")]
    async fn search(
        &self,
        Parameters(p): Parameters<SearchParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let results = db::search::global_search(&conn, &p.query).map_err(db_err)?;
            json_result(&results)
        })
    }

    // ── Write tools ──────────────────────────────────────────────────────

    #[tool(
        description = "Create a new task on a feature. For Jira-managed tasks, set source='jira' and provide external_key (e.g. PROJ-123), external_url, external_status, and description."
    )]
    async fn create_task(
        &self,
        Parameters(p): Parameters<CreateTaskParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        let title = p.title.clone();
        with_db!(self, conn => {
            let task = db::tasks::create_task(
                &conn,
                &fid,
                &title,
                p.source.as_deref(),
                p.external_key.as_deref(),
                p.external_url.as_deref(),
                p.external_status.as_deref(),
                p.description.as_deref(),
            ).map_err(db_err)?;
            let _ = config::push_notification(&format!("Task created: {}", title), Some(&fid));
            json_result(&task)
        })
    }

    #[tool(description = "Update a task's title, done status, external_status, or description")]
    async fn update_task(
        &self,
        Parameters(p): Parameters<UpdateTaskParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let task = db::tasks::update_task(
                &conn,
                &p.id,
                p.title.as_deref(),
                p.done,
                p.external_status.as_deref(),
                p.description.as_deref(),
            ).map_err(db_err)?;
            let _ = config::push_notification(&format!("Task updated: {}", task.title), Some(&task.feature_id));
            json_result(&task)
        })
    }

    #[tool(description = "Delete a task")]
    async fn delete_task(
        &self,
        Parameters(p): Parameters<DeleteTaskParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            db::tasks::delete_task(&conn, &p.id).map_err(db_err)?;
            let _ = config::push_notification("Task deleted", None);
            Ok(CallToolResult::success(vec![Content::text("Task deleted")]))
        })
    }

    #[tool(
        description = "Submit an implementation plan for user review in the FeatureHub UI. The plan body should be Markdown. Returns the created plan with status 'pending'. The user will review and approve/reject it in the GUI. Poll get_plan_status to check the outcome."
    )]
    async fn submit_plan(
        &self,
        Parameters(p): Parameters<SubmitPlanParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        let title = p.title.clone();
        // Auto-populate session_id from CLI if not provided
        let session_id = p.session_id.or_else(|| self.claude_session_id.clone());
        with_db!(self, conn => {
            // Auto-link session to feature if session_id provided and not already linked
            if let Some(ref sid) = session_id {
                let already_linked: bool = conn
                    .query_row(
                        "SELECT COUNT(*) FROM sessions WHERE feature_id = ?1 AND claude_session_id = ?2",
                        rusqlite::params![fid, sid],
                        |row| row.get::<_, i64>(0),
                    )
                    .unwrap_or(0) > 0;
                if !already_linked {
                    let _ = db::sessions::link_session(&conn, &fid, sid, None, None, None, None);
                }
            }

            let plan = db::plans::create_plan(
                &conn, &fid, session_id.as_deref(), &title, &p.body,
            ).map_err(db_err)?;
            let _ = config::push_notification_ex(
                &format!("Plan submitted for review: {}", title),
                Some(&fid),
                Some(&plan.id),
            );
            json_result(&plan)
        })
    }

    #[tool(
        description = "Check the status of a previously submitted plan. Returns the plan with its current status ('pending', 'approved', or 'rejected') and any feedback from the user."
    )]
    async fn get_plan_status(
        &self,
        Parameters(p): Parameters<GetPlanStatusParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let plan = db::plans::get_plan(&conn, &p.plan_id).map_err(db_err)?;
            json_result(&plan)
        })
    }

    #[tool(
        description = "Update an existing plan's title and/or body. If the plan was rejected, updating it resets the status to 'pending' for re-review. Use this instead of creating a new plan when revising a rejected plan."
    )]
    async fn update_plan(
        &self,
        Parameters(p): Parameters<UpdatePlanParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let plan_id = p.plan_id.clone();
        with_db!(self, conn => {
            let plan = db::plans::update_plan(
                &conn, &plan_id, p.title.as_deref(), p.body.as_deref(),
            ).map_err(db_err)?;
            let _ = config::push_notification(
                &format!("Plan updated: {}", plan.title), Some(&plan.feature_id)
            );
            json_result(&plan)
        })
    }

    #[tool(
        description = "Link the current Claude Code session to a feature. Call this at the start of your session so it appears in FeatureHub's session list. If the session is already linked, this is a no-op."
    )]
    async fn link_session(
        &self,
        Parameters(p): Parameters<LinkSessionParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        let csid = p.claude_session_id.clone();
        with_db!(self, conn => {
            // Check if already linked
            let existing: Option<String> = conn
                .query_row(
                    "SELECT id FROM sessions WHERE feature_id = ?1 AND claude_session_id = ?2",
                    rusqlite::params![fid, csid],
                    |row| row.get(0),
                )
                .ok();

            if existing.is_some() {
                return Ok(CallToolResult::success(vec![Content::text("Session already linked")]));
            }

            let session = db::sessions::link_session(
                &conn, &fid, &csid, p.title, None, p.project_path, None,
            ).map_err(db_err)?;
            let _ = config::push_notification("Session linked to feature", Some(&fid));
            json_result(&session)
        })
    }

    #[tool(
        description = "Move a session from one feature to another. Updates the session's feature association and search index."
    )]
    async fn move_session(
        &self,
        Parameters(p): Parameters<MoveSessionParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let sid = p.session_id.clone();
        let target_fid = p.target_feature_id.clone();
        with_db!(self, conn => {
            let session = db::sessions::move_session(&conn, &sid, &target_fid)
                .map_err(db_err)?;
            let _ = config::push_notification("Session moved to feature", Some(&target_fid));
            json_result(&session)
        })
    }

    #[tool(description = "Create or update the note for a feature")]
    async fn save_note(
        &self,
        Parameters(p): Parameters<SaveNoteParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        with_db!(self, conn => {
            let note = db::notes::save_note(&conn, &fid, &p.content).map_err(db_err)?;
            let _ = config::push_notification("Note updated", Some(&fid));
            json_result(&note)
        })
    }

    #[tool(
        description = "Get the persistent context/instructions for a feature. This context is injected into Claude sessions."
    )]
    async fn get_context(
        &self,
        Parameters(p): Parameters<FeatureIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let context = db::context::get_context(&conn, &p.feature_id).map_err(db_err)?;
            json_result(&context)
        })
    }

    #[tool(
        description = "Create or update persistent context/instructions for a feature. Use this for requirements, technical details, and session-spanning information that should persist across conversations."
    )]
    async fn save_context(
        &self,
        Parameters(p): Parameters<SaveContextParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        with_db!(self, conn => {
            let context = db::context::save_context(&conn, &fid, &p.content).map_err(db_err)?;
            let _ = config::push_notification("Context updated", Some(&fid));
            json_result(&context)
        })
    }

    #[tool(description = "Update a feature's status, title, or description")]
    async fn update_feature(
        &self,
        Parameters(p): Parameters<UpdateFeatureParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.id.clone();
        with_db!(self, conn => {
            // Handle parent_id change if provided
            if let Some(ref pid) = p.parent_id {
                let parent = if pid.is_empty() { None } else { Some(pid.clone()) };
                db::features::set_feature_parent(&conn, &fid, parent).map_err(db_err)?;
            }
            let feature = db::features::update_feature(
                &conn, &fid, p.title, None, p.status, None, p.description,
            ).map_err(db_err)?;
            let _ = config::push_notification(&format!("Feature updated: {}", feature.title), Some(&fid));
            json_result(&feature)
        })
    }

    #[tool(
        description = "Create a new feature. Returns the created feature with its ID. Optionally nest under a parent feature."
    )]
    async fn create_feature(
        &self,
        Parameters(p): Parameters<CreateFeatureParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let title = p.title.clone();
        with_db!(self, conn => {
            let feature = db::features::create_feature(
                &conn, &title, p.ticket_id, p.status, p.description, p.parent_id,
            ).map_err(db_err)?;
            let _ = config::push_notification(&format!("Feature created: {}", title), Some(&feature.id));
            json_result(&feature)
        })
    }

    #[tool(
        description = "Set or change a feature's parent. Pass a parent_id to nest under another feature, or pass an empty string / null to make it root-level. Prevents cycles."
    )]
    async fn set_feature_parent(
        &self,
        Parameters(p): Parameters<SetFeatureParentParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        with_db!(self, conn => {
            let parent = match p.parent_id {
                Some(ref pid) if pid.is_empty() => None,
                other => other,
            };
            let feature = db::features::set_feature_parent(&conn, &fid, parent).map_err(db_err)?;
            let _ = config::push_notification(&format!("Feature parent updated: {}", feature.title), Some(&fid));
            json_result(&feature)
        })
    }

    #[tool(
        description = "Delete a feature and its associated data (tasks, links, files, etc.). Children are moved to root level. This is irreversible."
    )]
    async fn delete_feature(
        &self,
        Parameters(p): Parameters<DeleteFeatureParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        let storage_base = config::get_active_db_path()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        with_db!(self, conn => {
            db::features::delete_feature(&conn, &fid, storage_base.as_deref()).map_err(db_err)?;
            let _ = config::push_notification("Feature deleted", None);
            Ok(CallToolResult::success(vec![Content::text("Feature deleted successfully")]))
        })
    }

    #[tool(
        description = "Toggle pin status on a feature. Pinned features sort to the top of the list."
    )]
    async fn toggle_pin(
        &self,
        Parameters(p): Parameters<TogglePinParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        with_db!(self, conn => {
            let feature = db::features::toggle_pin_feature(&conn, &fid).map_err(db_err)?;
            let status = if feature.pinned { "pinned" } else { "unpinned" };
            let _ = config::push_notification(&format!("Feature {}: {}", status, feature.title), Some(&fid));
            json_result(&feature)
        })
    }

    #[tool(description = "Archive or unarchive a feature")]
    async fn set_archived(
        &self,
        Parameters(p): Parameters<SetArchivedParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        with_db!(self, conn => {
            let feature = db::features::set_archived(&conn, &fid, p.archived).map_err(db_err)?;
            let action = if p.archived { "archived" } else { "unarchived" };
            let _ = config::push_notification(&format!("Feature {}: {}", action, feature.title), Some(&fid));
            json_result(&feature)
        })
    }

    #[tool(
        description = "Add a link to a feature. Returns an error if the URL already exists on this feature to prevent duplicates. For Jira links, include metadata with key, status, issue_type, summary, and assignee fields"
    )]
    async fn add_link(
        &self,
        Parameters(p): Parameters<AddLinkParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let fid = p.feature_id.clone();
        let title = p.title.clone();
        let metadata = p.metadata.clone();
        with_db!(self, conn => {
            // Duplicate URL check
            let existing = db::links::get_links(&conn, &fid).map_err(db_err)?;
            if let Some(dup) = existing.iter().find(|l| l.url == p.url) {
                return Err(rmcp::ErrorData::invalid_params(
                    format!("URL already exists on this feature as \"{}\" (id: {}). Use update_link to modify it instead.", dup.title, dup.id),
                    None,
                ));
            }
            let link = db::links::add_link(&conn, &fid, &title, &p.url, p.link_type, p.description).map_err(db_err)?;
            if let Some(ref meta) = metadata {
                let _ = db::links::update_link_metadata(&conn, &link.id, meta);
            }
            let _ = config::push_notification(&format!("Link added: {}", title), Some(&fid));
            json_result(&link)
        })
    }

    #[tool(
        description = "Update an existing link's title, URL, link_type, or description. Use get_links first to find the link ID."
    )]
    async fn update_link(
        &self,
        Parameters(p): Parameters<UpdateLinkParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let link = db::links::update_link(&conn, &p.id, p.title, p.url, p.link_type, p.description).map_err(db_err)?;
            let _ = config::push_notification(&format!("Link updated: {}", link.title), Some(&link.feature_id));
            json_result(&link)
        })
    }

    #[tool(description = "Delete a link from a feature. Use get_links first to find the link ID.")]
    async fn delete_link(
        &self,
        Parameters(p): Parameters<DeleteLinkParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            db::links::delete_link(&conn, &p.id).map_err(db_err)?;
            let _ = config::push_notification("Link deleted", None);
            Ok(CallToolResult::success(vec![Content::text("Link deleted")]))
        })
    }

    // ── Settings tools ────────────────────────────────────────────────────

    #[tool(
        description = "Get current app settings including fh CLI path, configured MCP servers, and default repositories"
    )]
    fn get_settings(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let global =
            config::load_settings().map_err(|e| rmcp::ErrorData::internal_error(e, None))?;
        let storage = config::load_storage_settings(&self.storage_path)
            .map_err(|e| rmcp::ErrorData::internal_error(e, None))?;
        // Return a merged view
        let combined = serde_json::json!({
            "fh_cli_path": global.fh_cli_path,
            "mcp_servers": storage.mcp_servers,
            "default_repositories": storage.default_repositories,
            "extensions": storage.extensions,
            "skills": storage.skills,
            "preferred_ides": global.preferred_ides,
        });
        json_result(&combined)
    }

    #[tool(
        description = "Update app settings. Can set the fh CLI path, configure MCP servers, and/or set default repositories. MCP servers and repositories are per-storage."
    )]
    fn save_settings(
        &self,
        Parameters(p): Parameters<SaveSettingsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        // Update global settings (fh_cli_path)
        if let Some(path) = p.fh_cli_path {
            let mut global = config::load_settings().unwrap_or_default();
            global.fh_cli_path = if path.is_empty() { None } else { Some(path) };
            config::save_settings(&global).map_err(|e| rmcp::ErrorData::internal_error(e, None))?;
        }
        // Update storage-specific settings (mcp_servers, default_repositories)
        let mut storage = config::load_storage_settings(&self.storage_path).unwrap_or_default();
        if let Some(servers) = p.mcp_servers {
            storage.mcp_servers = servers
                .into_iter()
                .map(|s| config::McpServer {
                    name: s.name,
                    command: s.command,
                    args: s.args,
                    env: s.env,
                    default_enabled: s.default_enabled.unwrap_or(true),
                    url: s.url,
                })
                .collect();
        }
        if let Some(repos) = p.default_repositories {
            storage.default_repositories = repos
                .into_iter()
                .map(|r| config::Repository {
                    url: r.url,
                    name: r.name,
                    description: r.description,
                })
                .collect();
        }
        config::save_storage_settings(&self.storage_path, &storage)
            .map_err(|e| rmcp::ErrorData::internal_error(e, None))?;
        json_result(&storage)
    }

    // ── Skills tools ──────────────────────────────────────────────────────

    #[tool(
        description = "List all configured skills. Skills are reusable instruction sets that can be enabled per-feature to inject guidelines into Claude sessions."
    )]
    fn get_skills(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let storage = config::load_storage_settings(&self.storage_path)
            .map_err(|e| rmcp::ErrorData::internal_error(e, None))?;
        json_result(&storage.skills)
    }

    #[tool(
        description = "Create a new skill. A skill is a reusable set of instructions/guidelines in Markdown that gets injected into Claude sessions when enabled for a feature. Use this to codify team practices, coding standards, or workflow guidelines."
    )]
    fn create_skill(
        &self,
        Parameters(p): Parameters<CreateSkillParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut storage = config::load_storage_settings(&self.storage_path).unwrap_or_default();
        if storage.skills.iter().any(|s| s.id == p.id) {
            return Err(rmcp::ErrorData::internal_error(
                format!(
                    "Skill with id '{}' already exists. Use update_skill to modify it.",
                    p.id
                ),
                None,
            ));
        }
        let skill = config::Skill {
            id: p.id,
            name: p.name,
            content: p.content,
            default_enabled: p.default_enabled.unwrap_or(true),
        };
        storage.skills.push(skill.clone());
        config::save_storage_settings(&self.storage_path, &storage)
            .map_err(|e| rmcp::ErrorData::internal_error(e, None))?;
        let _ = config::push_notification(&format!("Skill created: {}", skill.name), None);
        json_result(&skill)
    }

    #[tool(description = "Update an existing skill's name, content, or default_enabled status.")]
    fn update_skill(
        &self,
        Parameters(p): Parameters<UpdateSkillParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut storage = config::load_storage_settings(&self.storage_path).unwrap_or_default();
        let skill = storage
            .skills
            .iter_mut()
            .find(|s| s.id == p.id)
            .ok_or_else(|| {
                rmcp::ErrorData::internal_error(format!("Skill '{}' not found", p.id), None)
            })?;
        if let Some(name) = p.name {
            skill.name = name;
        }
        if let Some(content) = p.content {
            skill.content = content;
        }
        if let Some(enabled) = p.default_enabled {
            skill.default_enabled = enabled;
        }
        let updated = skill.clone();
        config::save_storage_settings(&self.storage_path, &storage)
            .map_err(|e| rmcp::ErrorData::internal_error(e, None))?;
        let _ = config::push_notification(&format!("Skill updated: {}", updated.name), None);
        json_result(&updated)
    }

    #[tool(description = "Delete a skill by its ID.")]
    fn delete_skill(
        &self,
        Parameters(p): Parameters<DeleteSkillParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut storage = config::load_storage_settings(&self.storage_path).unwrap_or_default();
        let before = storage.skills.len();
        storage.skills.retain(|s| s.id != p.id);
        if storage.skills.len() == before {
            return Err(rmcp::ErrorData::internal_error(
                format!("Skill '{}' not found", p.id),
                None,
            ));
        }
        config::save_storage_settings(&self.storage_path, &storage)
            .map_err(|e| rmcp::ErrorData::internal_error(e, None))?;
        let _ = config::push_notification("Skill deleted", None);
        Ok(CallToolResult::success(vec![Content::text(
            "Skill deleted",
        )]))
    }

    // ── Scoped convenience tool ──────────────────────────────────────────

    #[tool(description = "Get the current feature ID this session is scoped to (if any)")]
    fn get_current_feature(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        match &self.default_feature_id {
            Some(id) => Ok(CallToolResult::success(vec![Content::text(id.clone())])),
            None => Ok(CallToolResult::success(vec![Content::text(
                "No feature scoped — pass feature_id explicitly to tools",
            )])),
        }
    }

    #[tool(
        description = "Health check for MCP server diagnostics. Returns storage path, DB status, feature count, and extension info."
    )]
    fn health_check(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let db_status = match self.db.lock() {
            Ok(conn) => {
                let feature_count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM features", [], |row| row.get(0))
                    .unwrap_or(-1);
                let kb_count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM knowledge_entries", [], |row| {
                        row.get(0)
                    })
                    .unwrap_or(-1);
                serde_json::json!({
                    "connected": true,
                    "feature_count": feature_count,
                    "knowledge_entry_count": kb_count,
                })
            }
            Err(_) => serde_json::json!({ "connected": false, "error": "lock poisoned" }),
        };

        let extensions: Vec<_> = self
            .extension_registry
            .extensions
            .iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.manifest.id,
                    "name": e.manifest.name,
                    "enabled": e.enabled,
                    "tool_count": e.manifest.tools.len(),
                })
            })
            .collect();

        let result = serde_json::json!({
            "storage_path": self.storage_path.to_string_lossy(),
            "scoped_feature_id": self.default_feature_id,
            "claude_session_id": self.claude_session_id,
            "database": db_status,
            "extensions": extensions,
        });

        json_result(&result)
    }

    // ── Knowledge Base tools ────────────────────────────────────────────

    #[tool(
        description = "List knowledge base entries. Optionally filter by folder_id. Returns titles, descriptions, and IDs (not full content). Use get_knowledge_entry to read full content."
    )]
    async fn list_knowledge(
        &self,
        Parameters(p): Parameters<ListKnowledgeParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let entries = if let Some(ref fid) = p.folder_id {
                db::knowledge::list_entries_in_folder(&conn, Some(fid)).map_err(db_err)?
            } else {
                db::knowledge::list_entries(&conn).map_err(db_err)?
            };
            let folders = db::knowledge::list_folders(&conn).map_err(db_err)?;
            let result = serde_json::json!({ "entries": entries, "folders": folders });
            json_result(&result)
        })
    }

    #[tool(description = "Get full content of a knowledge base entry by ID")]
    async fn get_knowledge_entry(
        &self,
        Parameters(p): Parameters<KnowledgeEntryIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let entry = db::knowledge::get_entry(&conn, &p.id).map_err(db_err)?;
            json_result(&entry)
        })
    }

    #[tool(description = "Search knowledge base entries using full-text search")]
    async fn search_knowledge(
        &self,
        Parameters(p): Parameters<SearchParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let results = db::search::global_search(&conn, &p.query).map_err(db_err)?;
            let kb_results: Vec<_> = results.into_iter().filter(|r| r.entity_type == "knowledge").collect();
            json_result(&kb_results)
        })
    }

    #[tool(
        description = "Create a new knowledge base entry. Use this to save HOW-TOs, findings, research results, and other reusable knowledge."
    )]
    async fn create_knowledge_entry(
        &self,
        Parameters(p): Parameters<CreateKnowledgeEntryParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let entry = db::knowledge::create_entry(
                &conn,
                &p.title,
                &p.content,
                p.description.as_deref(),
                p.folder_id.as_deref(),
                p.feature_id.as_deref(),
            ).map_err(db_err)?;
            let _ = config::push_notification(&format!("Knowledge entry created: {}", entry.title), None);
            json_result(&entry)
        })
    }

    #[tool(description = "Update an existing knowledge base entry")]
    async fn update_knowledge_entry(
        &self,
        Parameters(p): Parameters<UpdateKnowledgeEntryParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let entry = db::knowledge::update_entry(
                &conn,
                &p.id,
                p.title.as_deref(),
                p.content.as_deref(),
                p.description.as_deref(),
                p.folder_id.as_ref().map(|o| o.as_deref()),
                p.feature_id.as_ref().map(|o| o.as_deref()),
            ).map_err(db_err)?;
            let _ = config::push_notification(&format!("Knowledge entry updated: {}", entry.title), None);
            json_result(&entry)
        })
    }

    #[tool(description = "Delete a knowledge base entry")]
    async fn delete_knowledge_entry(
        &self,
        Parameters(p): Parameters<KnowledgeEntryIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            db::knowledge::delete_entry(&conn, &p.id).map_err(db_err)?;
            Ok(CallToolResult::success(vec![Content::text("Knowledge entry deleted")]))
        })
    }

    #[tool(description = "Create a knowledge base folder for organizing entries")]
    async fn create_knowledge_folder(
        &self,
        Parameters(p): Parameters<CreateKnowledgeFolderParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let folder = db::knowledge::create_folder(&conn, &p.name, p.parent_id.as_deref()).map_err(db_err)?;
            let _ = config::push_notification(&format!("Knowledge folder created: {}", folder.name), None);
            json_result(&folder)
        })
    }

    #[tool(description = "Rename a knowledge base folder")]
    async fn rename_knowledge_folder(
        &self,
        Parameters(p): Parameters<RenameKnowledgeFolderParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let folder = db::knowledge::rename_folder(&conn, &p.id, &p.name).map_err(db_err)?;
            json_result(&folder)
        })
    }

    #[tool(
        description = "Delete a knowledge base folder. Entries inside are moved to the parent folder."
    )]
    async fn delete_knowledge_folder(
        &self,
        Parameters(p): Parameters<DeleteKnowledgeFolderParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            db::knowledge::delete_folder(&conn, &p.id).map_err(db_err)?;
            Ok(CallToolResult::success(vec![Content::text("Knowledge folder deleted")]))
        })
    }
}

impl ServerHandler for FeatureHubMcp {
    fn get_info(&self) -> ServerInfo {
        let instructions = match &self.feature_context {
            Some(ctx) => ctx.clone(),
            None => {
                let mut text = "FeatureHub MCP server. Use list_features to discover features, \
                    then pass feature_id to other tools.\n\n\
                    When asked to \"initialize\" a feature with inputs (links, files, descriptions), \
                    add all URLs via add_link (including child stories/tickets as individual links). \
                    Always search for related GitHub PRs and issues using the gh CLI — do NOT use web search for GitHub lookups. \
                    If a PR/issue URL is already provided, run gh pr view on it first to extract the ticket ID from the title or branch, \
                    then run gh search prs \"<TICKET-ID>\" (no --repo) to find all related PRs across all repos in one shot. \
                    Save context via save_context, set status via update_feature, and propose a better \
                    feature name if the current title is generic or unclear (place ticket ID at the beginning \
                    if present, e.g. \"HUB-123: User Auth Refactor\").\n\n\
                    Important: Tasks and Notes are user-owned. Do NOT create tasks or notes unless the user explicitly asks."
                    .to_string();

                // Append KB TOC for unscoped sessions too
                if let Ok(conn) = self.db.lock() {
                    let kb_entries = db::knowledge::list_entries(&conn).unwrap_or_default();
                    if !kb_entries.is_empty() {
                        text.push_str(&format!(
                            "\n\n## Knowledge Base ({} entries)\n",
                            kb_entries.len()
                        ));
                        text.push_str("Use get_knowledge_entry(id) to read full content.\n\n");
                        for entry in &kb_entries {
                            let folder_prefix = if let Some(ref fid) = entry.folder_id {
                                db::knowledge::get_folder_path(&conn, fid)
                                    .map(|p| format!("[{}/] ", p))
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            if entry.description.is_empty() {
                                text.push_str(&format!(
                                    "- {}{} (id: {})\n",
                                    folder_prefix, entry.title, entry.id
                                ));
                            } else {
                                text.push_str(&format!(
                                    "- {}{} — {} (id: {})\n",
                                    folder_prefix, entry.title, entry.description, entry.id
                                ));
                            }
                        }
                    }
                }

                text
            }
        };

        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(instructions)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        let mut tools = self.tool_router.list_all();

        for ext in &self.extension_registry.extensions {
            if !ext.enabled {
                continue;
            }
            for tool_decl in &ext.manifest.tools {
                tools.push(build_extension_tool(tool_decl));
            }
        }

        Ok(ListToolsResult {
            tools,
            meta: None,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let tool_name = request.name.as_ref();

        // Check extension tools first
        if let Some((ext, tool_decl)) = self.extension_registry.find_tool(tool_name) {
            let script_path = ext.dir.join(&tool_decl.handler);
            let db_path = self
                .storage_path
                .join("feature-hub.db")
                .to_string_lossy()
                .to_string();
            let sp = self.storage_path.to_string_lossy().to_string();
            let params = request.arguments.unwrap_or_default();
            let input = extensions::script_runner::ScriptInput {
                params,
                db_path,
                storage_path: sp,
                feature_id: self.default_feature_id.clone(),
            };
            let timeout = tool_decl.timeout_secs.unwrap_or(10);
            return tokio::task::spawn_blocking(move || {
                extensions::script_runner::run_blocking(&script_path, &input, timeout)
            })
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?
            .map(|data| {
                let text = serde_json::to_string(&data).unwrap_or_default();
                CallToolResult::success(vec![Content::text(text)])
            })
            .map_err(|e| rmcp::ErrorData::internal_error(e, None));
        }

        // Fall through to static tool router
        let tool_context =
            rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context).await
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        if let Some((_, tool_decl)) = self.extension_registry.find_tool(name) {
            return Some(build_extension_tool(tool_decl));
        }
        self.tool_router.get(name).cloned()
    }
}

/// Build an MCP `Tool` from an extension manifest `ToolDecl`.
fn build_extension_tool(tool_decl: &feature_hub::extensions::manifest::ToolDecl) -> Tool {
    let mut props = serde_json::Map::new();
    let mut required_params: Vec<String> = Vec::new();
    for (param_name, param_decl) in &tool_decl.params {
        let mut prop = serde_json::json!({ "type": param_decl.param_type });
        if let Some(ref desc) = param_decl.description {
            prop["description"] = serde_json::Value::String(desc.clone());
        }
        props.insert(param_name.clone(), prop);
        if param_decl.required {
            required_params.push(param_name.clone());
        }
    }
    let schema = serde_json::json!({
        "type": "object",
        "properties": props,
        "required": required_params
    });
    let input_schema = std::sync::Arc::new(schema.as_object().cloned().unwrap_or_default());
    Tool::new(
        tool_decl.name.clone(),
        tool_decl.description.clone(),
        input_schema,
    )
}

// ─── Feature context builder ─────────────────────────────────────────────────

/// Max characters for context/note content in the server instructions snapshot.
/// Full content is always available via get_context / get_note tools.
const CONTEXT_SNAPSHOT_LIMIT: usize = 5000;

/// Truncate text at a paragraph boundary (double newline) near the limit.
fn truncate_at_paragraph(text: &str, limit: usize) -> (&str, bool) {
    if text.len() <= limit {
        return (text, false);
    }
    // Search backwards from limit for a paragraph break
    if let Some(pos) = text[..limit].rfind("\n\n") {
        (&text[..pos], true)
    } else if let Some(pos) = text[..limit].rfind('\n') {
        (&text[..pos], true)
    } else {
        (&text[..limit], true)
    }
}

fn build_feature_context(
    conn: &Connection,
    feature_id: &str,
    claude_session_id: Option<&str>,
    storage_path: &std::path::Path,
) -> Result<String, String> {
    let feature = db::features::get_feature(conn, feature_id)?;
    let tasks = db::tasks::get_tasks(conn, feature_id)?;
    let note = db::notes::get_note(conn, feature_id)?;
    let context = db::context::get_context(conn, feature_id)?;
    let links = db::links::get_links(conn, feature_id)?;
    let dirs = db::directories::get_directories(conn, feature_id)?;
    let plans = db::plans::get_plans(conn, feature_id)?;
    let sessions = db::sessions::get_sessions(conn, feature_id).unwrap_or_default();
    let files = db::files::get_files(conn, feature_id).unwrap_or_default();

    let mut ctx = String::new();
    let now = chrono::Utc::now().to_rfc3339();
    ctx.push_str(&format!(
        "FeatureHub MCP server — session scoped to this feature (loaded at {}):\n\n",
        now
    ));
    ctx.push_str("NOTE: This is a live snapshot. If you are resuming a session, this data is FRESH from the database and may differ from what you saw previously. Always trust this current state over your prior conversation context.\n\n");

    // Feature summary
    ctx.push_str(&format!("# {} ({})\n", feature.title, feature.status));
    ctx.push_str(&format!("Feature ID: {}\n", feature.id));
    if let Some(sid) = claude_session_id {
        ctx.push_str(&format!("Claude Session ID: {}\n", sid));
    }
    if let Some(desc) = &feature.description {
        if !desc.is_empty() {
            ctx.push_str(&format!("Description: {}\n", desc));
        }
    }
    if let Some(ticket) = &feature.ticket_id {
        ctx.push_str(&format!("Ticket: {}\n", ticket));
    }
    if !feature.tags.is_empty() {
        let tag_names: Vec<&str> = feature.tags.iter().map(|t| t.name.as_str()).collect();
        ctx.push_str(&format!("Tags: {}\n", tag_names.join(", ")));
    }
    if let Some(ref pid) = feature.parent_id {
        if let Ok(parent) = db::features::get_feature(conn, pid) {
            ctx.push_str(&format!("Parent: {} (id: {})\n", parent.title, parent.id));
        }
    }
    // Show children (targeted query instead of loading all features)
    let children = db::features::get_feature_children(conn, feature_id).unwrap_or_default();
    if !children.is_empty() {
        ctx.push_str(&format!("Children ({}):", children.len()));
        for (child_id, child_title) in &children {
            ctx.push_str(&format!(" {} ({}),", child_title, child_id));
        }
        ctx.push('\n');
    }

    // Repositories
    if !dirs.is_empty() {
        ctx.push_str("\n## Repositories\n");
        ctx.push_str(
            "IMPORTANT: These are the cloned code repositories for this feature. \
Your current working directory is a FeatureHub workspace, NOT the code repo. \
Always use these paths as the base for reading, editing, and searching code files.\n\n",
        );
        for d in &dirs {
            let status = d.clone_status.as_deref().unwrap_or("ready");
            let label = d.label.as_deref().unwrap_or("");
            let url_info = d.repo_url.as_deref().unwrap_or("");
            let resolved_path = feature_hub::paths::resolve_path_string(&d.path, storage_path);
            match status {
                "ready" => {
                    let branch =
                        feature_hub::git::get_current_branch(std::path::Path::new(&resolved_path))
                            .ok();
                    if !label.is_empty() {
                        ctx.push_str(&format!("- {} ({})", resolved_path, label));
                    } else {
                        ctx.push_str(&format!("- {}", resolved_path));
                    }
                    if let Some(ref b) = branch {
                        ctx.push_str(&format!(" [branch: {}]", b));
                    }
                    if !url_info.is_empty() {
                        ctx.push_str(&format!(" (remote: {})", url_info));
                    }
                    ctx.push('\n');
                }
                "cloning" => {
                    ctx.push_str(&format!(
                        "- {} [CLONING - not ready yet] ({})\n",
                        label, url_info
                    ));
                }
                "failed" => {
                    let err = d.clone_error.as_deref().unwrap_or("unknown error");
                    ctx.push_str(&format!("- {} [FAILED: {}] ({})\n", label, err, url_info));
                }
                _ => {
                    ctx.push_str(&format!("- {} ({})\n", resolved_path, status));
                }
            }
        }
    }

    // Tasks
    if !tasks.is_empty() {
        let done_count = tasks.iter().filter(|t| t.done).count();
        ctx.push_str(&format!("\n## Tasks ({}/{})\n", done_count, tasks.len()));
        for t in &tasks {
            let check = if t.done { "x" } else { " " };
            let mut line = format!("- [{}] {}", check, t.title);
            if let Some(ref key) = t.external_key {
                line.push_str(&format!(" [{}]", key));
            }
            if let Some(ref status) = t.external_status {
                line.push_str(&format!(" ({})", status));
            }
            line.push_str(&format!(" (id: {})", t.id));
            ctx.push_str(&line);
            ctx.push('\n');
            if let Some(ref desc) = t.description {
                if !desc.is_empty() {
                    let truncated = if desc.len() > 200 {
                        &desc[..200]
                    } else {
                        desc.as_str()
                    };
                    ctx.push_str(&format!("  > {}\n", truncated.replace('\n', " ")));
                }
            }
        }
    }

    // Pending plans
    let pending_plans: Vec<_> = plans.iter().filter(|p| p.status == "pending").collect();
    if !pending_plans.is_empty() {
        ctx.push_str(&format!("\n## Pending Plans ({})\n", pending_plans.len()));
        ctx.push_str("These plans are awaiting user review in the FeatureHub UI. Use `get_plan_status` to check if they've been resolved.\n");
        for p in &pending_plans {
            ctx.push_str(&format!(
                "- {} (id: {}, submitted: {})\n",
                p.title, p.id, p.created_at
            ));
        }
    }

    // Links
    if !links.is_empty() {
        ctx.push_str("\n## Links\n");
        for l in &links {
            if let Some(desc) = &l.description {
                ctx.push_str(&format!(
                    "- [{}]({}) ({}) — {}\n",
                    l.title, l.url, l.link_type, desc
                ));
            } else {
                ctx.push_str(&format!("- [{}]({}) ({})\n", l.title, l.url, l.link_type));
            }
        }
    }

    // Session history
    if !sessions.is_empty() {
        let other_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| claude_session_id.is_none_or(|cid| s.claude_session_id != cid))
            .collect();
        if !other_sessions.is_empty() {
            ctx.push_str(&format!(
                "\n## Session History ({} previous)\n",
                other_sessions.len()
            ));
            if let Some(last) = other_sessions.first() {
                let date = last.ended_at.as_deref().unwrap_or(last.linked_at.as_str());
                let title = last.title.as_deref().unwrap_or("untitled");
                ctx.push_str(&format!(
                    "Most recent: \"{}\" ({})\n",
                    title,
                    &date[..10.min(date.len())]
                ));
            }
        }
    }

    // Files
    if !files.is_empty() {
        ctx.push_str(&format!("\n## Files ({} uploaded)\n", files.len()));
        ctx.push_str("Use `get_files` tool to access file details and paths.\n");
        // Build folder path map for display
        let folders = db::folders::get_folders(conn, feature_id).unwrap_or_default();
        let folder_names: std::collections::HashMap<&str, &str> = folders
            .iter()
            .map(|f| (f.id.as_str(), f.name.as_str()))
            .collect();
        for f in files.iter().take(20) {
            let size_kb = f.size / 1024;
            let prefix = f
                .folder_id
                .as_deref()
                .and_then(|fid| folder_names.get(fid))
                .map(|name| format!("{}/", name))
                .unwrap_or_default();
            ctx.push_str(&format!("- {}{} ({}KB)\n", prefix, f.filename, size_kb));
        }
        if files.len() > 20 {
            ctx.push_str(&format!("...and {} more\n", files.len() - 20));
        }
    }

    // Context
    if let Some(c) = &context {
        if !c.content.is_empty() {
            ctx.push_str("\n## Context\n");
            let (text, truncated) = truncate_at_paragraph(&c.content, CONTEXT_SNAPSHOT_LIMIT);
            ctx.push_str(text);
            if truncated {
                ctx.push_str("\n...(truncated — use `get_context` tool for full content)\n");
            } else {
                ctx.push('\n');
            }
        }
    }

    // Note
    if let Some(n) = &note {
        if !n.content.is_empty() {
            ctx.push_str("\n## Notes\n");
            let (text, truncated) = truncate_at_paragraph(&n.content, CONTEXT_SNAPSHOT_LIMIT);
            ctx.push_str(text);
            if truncated {
                ctx.push_str("\n...(truncated — use `get_note` tool for full content)\n");
            } else {
                ctx.push('\n');
            }
        }
    }

    ctx.push_str(
        "\nUse `get_current_feature` to retrieve the feature ID, then pass it to other tools. \
Use `search` to find related features, links, or sessions across the entire storage.\n",
    );

    // Add initialization instructions
    ctx.push_str("\n## Feature Initialization\n");
    ctx.push_str("When the user asks to \"initialize\" a feature and provides inputs (links, files, descriptions, context), \
populate the feature using the available tools:\n");
    ctx.push_str("1. **Links**: Use `add_link` for every URL provided (docs, repos, designs, tickets, references). Auto-detect the link type from the URL. \
Use the `description` field to annotate each link's role (e.g. \"Epic\", \"Story\", \"Design doc\", \"API spec\"). \
When the input is an epic or parent ticket, add its child stories/tickets as individual links (type: `jira`) — do NOT create tasks for them.\n\
   **Finding GitHub links**: Always search for related GitHub PRs and issues using the `gh` CLI. \
Do NOT use web search for GitHub lookups. Efficient search order: \
(a) If a GitHub PR or issue URL is already provided, run `gh pr view <number> --repo <owner>/<repo>` immediately to get its title and branch name. \
(b) Extract the ticket ID from the PR title or branch name (e.g. \"YLD-7233\" from \"feature/YLD-7233-add-file-versioning\"). \
(c) Run `gh search prs \"<TICKET-ID>\"` (no --repo flag) to find all related PRs across every repository in one shot. \
Do NOT waste time with `gh pr list` when you already have a ticket ID or a direct PR URL.\n\
   For **Jira links**, always include the `metadata` field with available info: \
`{\"key\": \"HUB-123\", \"status\": \"To Do\", \"issue_type\": \"Story\", \"summary\": \"Short issue summary\", \"assignee\": \"Person Name\"}`. \
This metadata is displayed in the UI as a rich issue view with status badges and assignee. Omit fields you don't have.\n\
   **Jira story descriptions**: If a Jira story has an empty or missing description, write a concise high-level plan \
for it — what needs to happen and why, not exact implementation details. Keep it short and actionable. \
However, if the feature is already done and the user asks you to update the Jira description, then include \
implementation details describing how it was actually implemented.\n");
    ctx.push_str("2. **Context**: Use `save_context` for persistent requirements, technical details, and session-spanning information. \
This is the most important step — context is the first thing future Claude sessions see and must be thorough.\n");
    ctx.push_str("3. **Repositories**: Use `get_default_repositories` to see available repository URLs (with descriptions). \
Propose which ones are relevant to this feature and let the user confirm before cloning them with `clone_repository`.\n");
    ctx.push_str("4. **Status**: Use `update_feature` to set an appropriate status (e.g. `in_progress` if work is starting).\n");
    ctx.push_str("5. **Feature Name**: After gathering all context, evaluate if the feature's current title accurately reflects \
the work. If the title is generic, unclear, or doesn't match the actual scope, propose a better name to the user. \
Keep names concise but descriptive. If a ticket ID exists, place it at the beginning (e.g. \"HUB-123: User Auth Refactor\" not just \"HUB-123\" or \"New Feature\"). \
Use `update_feature` with the `title` field once the user confirms.\n");
    ctx.push_str("\n**Important**: Tasks and Notes are user-owned. Do NOT create tasks or notes during initialization \
or at any other time unless the user explicitly asks you to. Tasks are for the user to manage their own TODOs. \
Notes are for the user to write manually. Even though you have access to these tools, only use `create_task`, \
`save_note` when the user specifically requests it.\n\
However, when you complete work that directly corresponds to an existing task, ask the user if they'd like you to mark it done via `update_task`.\n");

    // Context maintenance instructions
    ctx.push_str("\n## Keeping Context Up to Date\n");
    ctx.push_str("**CRITICAL**: The Context is the single most important artifact of a session. \
It is persistent across sessions — it's the first thing future Claude sessions see, and it's the only way \
knowledge survives between sessions. You MUST keep it current. Use `save_context` whenever:\n");
    ctx.push_str("- You make architectural decisions or significant implementation choices\n");
    ctx.push_str("- Requirements change or are clarified during the session\n");
    ctx.push_str("- You discover important technical details, constraints, or gotchas\n");
    ctx.push_str("- The approach or plan evolves from what the context currently describes\n");
    ctx.push_str("- You finish a milestone or the session ends — summarize where things stand\n");
    ctx.push_str("\nThink of context as a living briefing document. Read it at the start, update it as you go, \
and leave it accurate for the next session. Do NOT duplicate task lists or notes — context is for \
high-level decisions, architecture, requirements, and current state.\n\
**Do not end a session without updating context if anything meaningful happened.**\n");

    // Custom commands
    ctx.push_str("\n## Commands\n");
    ctx.push_str("When the user says one of these commands, follow the described behavior:\n\n");
    ctx.push_str("**\"refresh feature\"** — Re-sync the feature with external sources:\n");
    ctx.push_str("1. Check the feature's links for any Jira/ticket URLs\n");
    ctx.push_str("2. If Jira MCP or similar tools are available, fetch the latest status of linked stories/epics\n");
    ctx.push_str("3. Update link titles or descriptions if they've changed\n");
    ctx.push_str("4. Update the feature context with any new information (status changes, new requirements, blockers)\n");
    ctx.push_str("5. Report what changed to the user\n");

    // Ongoing behaviors
    ctx.push_str("\n## Ongoing Behaviors\n");
    ctx.push_str("These apply throughout the session without the user needing to ask:\n\n");
    if claude_session_id.is_none() {
        ctx.push_str("**Link your session** — At the very start of this session, call `link_session` with this feature's ID \
and your Claude Code session ID to register yourself in FeatureHub. This ensures your session appears in the feature's session list.\n\n");
    }
    ctx.push_str(
        "**Validate branch before working** — Before implementation, run `git status`. \
Alert the user if there are uncommitted changes. If clean, check for the feature branch — \
propose switching or creating one. Always confirm before switching/creating branches.\n\n",
    );

    ctx.push_str("**Save PR links** — After creating a pull request, save its URL with `add_link` (type `github`, description \"PR: <title>\").\n\n");

    ctx.push_str("**Before ending a session** — Update the feature context (`save_context`) with where things stand: \
what was accomplished, what's left to do, any decisions made, and any blockers. \
If the feature status should change (e.g. `in_progress` → `in_review`), update it with `update_feature`. \
Leave the feature in a state where the next session (or the user) can pick up immediately.\n");

    // Plan submission instructions
    ctx.push_str("\n## Plan Submission\n");
    ctx.push_str("For significant implementation plans, use `submit_plan` with a clear title and detailed Markdown body. \
Poll with `get_plan_status` and wait for approval. If rejected, use `update_plan` to revise (not create a new one).\n");

    // Git rules
    ctx.push_str("\n## Git Rules\n");
    ctx.push_str("- Only use git commands (log, diff, blame) to validate changes — not to understand the code. Read source files directly. \
If you're missing context, ask the user.\n");
    ctx.push_str("- Never mention \"Claude\" or AI tools in commit messages — keep them about what changed.\n");

    // Knowledge Base TOC — storage-scoped, always included if entries exist
    let kb_entries = db::knowledge::list_entries(conn).unwrap_or_default();
    if !kb_entries.is_empty() {
        ctx.push_str(&format!(
            "\n## Knowledge Base ({} entries)\n",
            kb_entries.len()
        ));
        ctx.push_str("Use `get_knowledge_entry(id)` to read full content, or `search_knowledge` to search by keyword. \
Check relevant entries before starting implementation — they may contain HOW-TOs, patterns, or prior research.\n\
If you discover a reusable pattern, gotcha, or technique during this session, consider saving it with `create_knowledge_entry` \
so it's available across all features.\n\n");

        for entry in &kb_entries {
            let folder_prefix = if let Some(ref fid) = entry.folder_id {
                if let Ok(path) = db::knowledge::get_folder_path(conn, fid) {
                    format!("[{}/] ", path)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            if entry.description.is_empty() {
                ctx.push_str(&format!(
                    "- {}{} (id: {})\n",
                    folder_prefix, entry.title, entry.id
                ));
            } else {
                ctx.push_str(&format!(
                    "- {}{} — {} (id: {})\n",
                    folder_prefix, entry.title, entry.description, entry.id
                ));
            }
        }
    }

    // Extension instructions (conditional on each extension being enabled)
    let storage_settings = config::load_storage_settings(storage_path).unwrap_or_default();
    for ext in &storage_settings.extensions {
        if ext.enabled && !ext.instructions.is_empty() {
            ctx.push('\n');
            ctx.push_str(&ext.instructions);
            ctx.push('\n');
        }
    }

    // Skill instructions (resolved per-feature with overrides)
    let active_skills =
        db::skills::resolve_skills_for_feature(conn, feature_id, &storage_settings.skills)
            .unwrap_or_default();
    for skill in &active_skills {
        if !skill.content.is_empty() {
            ctx.push_str(&format!("\n## Skill: {}\n", skill.name));
            ctx.push_str(&skill.content);
            ctx.push('\n');
        }
    }

    Ok(ctx)
}

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Open the active storage
    let storage_path = config::get_active_storage_path()?;
    let db_path = storage_path.join("feature-hub.db");
    if !db_path.exists() {
        return Err(format!(
            "Database not found at {}. Open FeatureHub to create a storage first.",
            db_path.display()
        )
        .into());
    }
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db::initialize(&conn).map_err(|e| format!("Failed to initialize database: {}", e))?;
    db::migrate_to_relative_paths(&conn, &storage_path);

    let service = FeatureHubMcp::new(conn, cli.feature, cli.session_id, storage_path);

    let server = service.serve(stdio()).await?;
    server.waiting().await?;

    Ok(())
}
