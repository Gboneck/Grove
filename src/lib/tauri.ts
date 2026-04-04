import { invoke } from "@tauri-apps/api/core";

export interface Block {
  type: string;
  [key: string]: unknown;
}

export interface ReasonResponse {
  blocks: Block[];
  timestamp: string;
  model_source: "local" | "cloud";
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
