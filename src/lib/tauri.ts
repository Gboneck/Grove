import { invoke } from "@tauri-apps/api/core";

export interface Block {
  type: string;
  [key: string]: unknown;
}

export interface ReasonResponse {
  blocks: Block[];
  timestamp: string;
  model_source: "local" | "cloud";
  ambient_mood: string | null;
  theme_hint: string | null;
  conversation_id: string;
  auto_action_results: string[];
  venture_update_results: string[];
}

export async function reason(userInput?: string): Promise<ReasonResponse> {
  return invoke<ReasonResponse>("reason", {
    userInput: userInput || null,
  });
}

export interface ModelStatus {
  gemma_available: boolean;
  claude_available: boolean;
  mode: string;
  local_model: string;
  cloud_model: string;
}

export async function getModelStatus(): Promise<ModelStatus> {
  return invoke<ModelStatus>("get_model_status");
}

export async function setModelMode(mode: "auto" | "local_only" | "cloud_only"): Promise<void> {
  return invoke<void>("set_model_mode", { mode });
}

export interface SetupStatus {
  grove_dir_exists: boolean;
  soul_md_exists: boolean;
  soul_md_is_default: boolean;
  api_key_set: boolean;
  ollama_available: boolean;
  needs_setup: boolean;
  local_model: string;
  system_ram_gb: number;
  recommended_model: string;
}

export async function checkSetup(): Promise<SetupStatus> {
  return invoke<SetupStatus>("check_setup");
}

export async function saveApiKey(key: string): Promise<void> {
  return invoke<void>("save_api_key", { key });
}

export async function readSoul(): Promise<string> {
  return invoke<string>("read_soul");
}

export async function writeSoul(content: string): Promise<void> {
  return invoke<void>("write_soul", { content });
}

export async function readContext(): Promise<unknown> {
  return invoke<unknown>("read_context");
}

export async function writeContext(context: unknown): Promise<void> {
  return invoke<void>("write_context", { context });
}

export interface SystemInfo {
  current_time: string;
  day_of_week: string;
  date: string;
  hostname: string;
}

export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>("get_system_info");
}

export async function getReasoningLogs(date?: string): Promise<unknown[]> {
  return invoke<unknown[]>("get_reasoning_logs", { date: date || null });
}

export interface FileStamps {
  files: Record<string, number>;
}

export async function getFileStamps(): Promise<FileStamps> {
  return invoke<FileStamps>("get_file_stamps");
}

// Identity wizard
export async function generateSoul(
  name: string,
  location: string | null,
  role: string | null,
  projects: string[],
  priorities: string[],
  workStyle: string | null
): Promise<string> {
  return invoke<string>("generate_soul", {
    name,
    location,
    role,
    projects,
    priorities,
    workStyle,
  });
}

export async function isSoulPersonalized(): Promise<boolean> {
  return invoke<boolean>("is_soul_personalized");
}

// Actions
export interface ActionDef {
  id: string;
  label: string;
  description: string;
  executor: string;
}

export interface ActionResult {
  success: boolean;
  message: string;
  output: string | null;
}

export async function executeAction(
  actionId: string,
  params?: Record<string, unknown>
): Promise<ActionResult> {
  return invoke<ActionResult>("execute_action", {
    actionId,
    params: params || null,
  });
}

export async function listActions(): Promise<ActionDef[]> {
  return invoke<ActionDef[]>("list_actions");
}

// Memory stats
export interface MemoryStats {
  total_sessions: number;
  total_facts: number;
  total_patterns: number;
  total_insights: number;
}

export async function getMemoryStats(): Promise<MemoryStats> {
  return invoke<MemoryStats>("get_memory_stats");
}

export async function recordActionEngagement(
  blockType: string,
  interacted: boolean
): Promise<void> {
  return invoke<void>("record_action_engagement", { blockType, interacted });
}

// Full memory (for viewer panel)
export async function getFullMemory(): Promise<unknown> {
  return invoke<unknown>("get_full_memory");
}

// Conversation management
export async function clearConversation(): Promise<void> {
  return invoke<void>("clear_conversation");
}

// Weekly digest
export async function getWeeklyDigest(): Promise<unknown> {
  return invoke<unknown>("get_weekly_digest");
}

// Roles
export interface Role {
  name: string;
  display: string;
  description: string;
  system_prompt_prefix: string;
  block_preferences: string[];
  avoid_blocks: string[];
  autonomy_level: string;
}

export async function listRoles(): Promise<Role[]> {
  return invoke<Role[]>("list_roles");
}

export async function getActiveRole(): Promise<string | null> {
  return invoke<string | null>("get_active_role");
}

export async function setActiveRole(name: string | null): Promise<void> {
  return invoke<void>("set_active_role", { name });
}

// Streaming reasoning — kicks off reason_stream which emits events
export async function reasonStream(userInput?: string): Promise<ReasonResponse> {
  return invoke<ReasonResponse>("reason_stream", {
    userInput: userInput || null,
  });
}
