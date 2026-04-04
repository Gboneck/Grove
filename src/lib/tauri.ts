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
