// ── Data Types ──────────────────────────────────────────────────────────

export interface Feature {
  id: string;
  title: string;
  description: string | null;
  ticket_id: string | null;
  status: string;
  pinned: boolean;
  archived: boolean;
  parent_id: string | null;
  group_id: string | null;
  created_at: string;
  updated_at: string;
  sort_order: number;
  tags?: Tag[];
  links?: Link[];
  directories?: Directory[];
  task_count_total?: number;
  task_count_done?: number;
}

export interface Tag {
  id: string;
  name: string;
  color: string;
}

export interface Link {
  id: string;
  feature_id: string;
  title: string;
  url: string;
  link_type: string;
  description: string | null;
  metadata: Record<string, any> | null;
  created_at: string;
}

export interface Directory {
  id: string;
  feature_id: string;
  path: string;
  label: string | null;
  repo_url: string | null;
  clone_status: string | null;
  clone_error: string | null;
}

export interface FeatureGroup {
  id: string;
  name: string;
  color: string | null;
  sort_order: number;
  created_at: string;
}

export interface DetectedIde {
  id: string;
  name: string;
  command: string;
}

export interface FileEntry {
  id: string;
  feature_id: string;
  filename: string;
  original_path: string;
  stored_path: string;
  size: number;
  folder_id: string | null;
  created_at: string;
}

export interface Folder {
  id: string;
  feature_id: string;
  parent_id: string | null;
  name: string;
  created_at: string;
}

export interface FilePreview {
  file_id: string;
  preview_type: "text" | "image" | "binary";
  content: string | null;
  mime_type: string | null;
  truncated: boolean;
}

export interface Session {
  id: string;
  feature_id: string;
  claude_session_id: string | null;
  title: string | null;
  summary: string | null;
  project_path: string | null;
  branch: string | null;
  started_at: string | null;
  ended_at: string | null;
  duration_mins: number | null;
  turns: number | null;
}

export interface Task {
  id: string;
  feature_id: string;
  title: string;
  done: boolean;
  sort_order: number;
  created_at: string;
  source: string;
  external_key: string | null;
  external_url: string | null;
  external_status: string | null;
  description: string | null;
}

export interface Note {
  id: string;
  feature_id: string;
  content: string;
  updated_at: string;
}

export interface Plan {
  id: string;
  feature_id: string;
  session_id: string | null;
  title: string;
  body: string;
  status: string;
  feedback: string | null;
  created_at: string;
  resolved_at: string | null;
}

export interface SearchResult {
  entity_type: string;
  entity_id: string;
  feature_id: string;
  title: string;
  snippet: string;
}

export interface FeatureData {
  feature: Feature;
  all_tags: Tag[];
  tasks: Task[];
  plans: Plan[];
  note: Note | null;
}

export interface GitStatusSummary {
  branch: string;
  modified: number;
  untracked: number;
  staged: number;
  ahead: number | null;
  behind: number | null;
}

export interface StorageInfo {
  id: string;
  name: string;
  path: string;
  is_active: boolean;
  git_status: "clean" | "dirty" | "none";
  icon: string | null;
}

export interface CliInstallResult {
  install_dir: string;
  binaries: string[];
  path_updated: boolean;
}

export interface McpServer {
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  default_enabled: boolean;
  url?: string | null;
}

export interface FeatureMcpServer {
  server_name: string;
  enabled: boolean;
}

export interface Repository {
  url: string;
  name?: string | null;
  description?: string | null;
}

export interface Extension {
  id: string;
  enabled: boolean;
  mcp_server: McpServer | null;
  instructions: string;
}

export interface Skill {
  id: string;
  name: string;
  content: string;
  default_enabled: boolean;
}

export interface FeatureSkill {
  skill_id: string;
  enabled: boolean;
}

export interface AppSettings {
  fh_cli_path: string | null;
  mcp_servers: McpServer[];
  default_repositories: Repository[];
  mermaid_diagrams: boolean;
  openfga_highlighting: boolean;
  show_tab_emojis: boolean;
  ui_font: string | null;
  mono_font: string | null;
  ui_font_size: number | null;
  terminal_font_size: number | null;
  extensions: Extension[];
  preferred_ides: string[];
  skills: Skill[];
}

export interface AppNotification {
  message: string;
  feature_id: string | null;
  plan_id?: string | null;
  timestamp: string;
}

export interface SessionActivity {
  counts: Record<string, number>;
  active_session_ids: string[];
}

export interface PanelSession {
  // Identity
  id: string;
  feature_id: string;
  feature_name: string;
  claude_session_id: string;

  // Display
  title: string;
  title_source: TitleSource;
  branch: string | null;

  // Time
  last_activity: string;
  started_at: string | null;
  ended_at: string | null;

  // Status
  is_active: boolean;
  status: SessionStatus;
  last_action: string | null;

  // Stats
  model: string | null;
  total_tokens: number | null;
  context_tokens: number | null;
  cost_usd: number | null;
}

export interface SessionsPanelData {
  sessions: PanelSession[];
  active_count: number;
}

export type SessionStatus = 'Active' | 'WaitingForInput' | 'Idle' | 'Finished' | 'Lost';

export type TitleSource = 'SessionsIndex' | 'SessionMemory' | 'FirstPrompt' | 'FeatureName' | 'Default';

export interface PtySessionResult {
  terminalId: string;
  sessionDbId: string;
  claudeSessionId?: string;
}

export interface Context {
  id: string;
  feature_id: string;
  content: string;
  updated_at: string;
}

export interface TimelineEvent {
  event_type: string;
  title: string;
  detail: string | null;
  timestamp: string;
}

export interface KnowledgeFolder {
  id: string;
  parent_id: string | null;
  name: string;
  sort_order: number;
  created_at: string;
}

export interface KnowledgeEntry {
  id: string;
  folder_id: string | null;
  feature_id: string | null;
  title: string;
  description: string;
  content: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface InstalledExtensionParamDecl {
  type: string;
  required: boolean;
  description?: string | null;
}

export interface InstalledExtensionToolDecl {
  name: string;
  description: string;
  handler: string;
  params: Record<string, InstalledExtensionParamDecl>;
  timeout_secs?: number | null;
}

export interface InstalledExtensionTabDecl {
  id: string;
  label: string;
  emoji: string;
  sort_order: number;
  component: string;
  badge_query?: string | null;
}

export interface InstalledExtensionManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  requires: string[];
  tools: InstalledExtensionToolDecl[];
  tabs: InstalledExtensionTabDecl[];
  storage_settings_key?: string;
  instructions: string;
}

export interface RequiresStatusInfo {
  name: string;
  found: boolean;
  path: string | null;
}

export interface ExtensionInfo {
  manifest: InstalledExtensionManifest;
  enabled: boolean;
  dir: string;
  requires_status: RequiresStatusInfo[];
}
