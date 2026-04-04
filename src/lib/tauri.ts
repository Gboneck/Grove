import { invoke } from "@tauri-apps/api/core";

export interface Block {
  type: string;
  [key: string]: unknown;
}

export interface ReasonResponse {
  blocks: Block[];
  timestamp: string;
}

export async function reason(userInput?: string): Promise<ReasonResponse> {
  return invoke<ReasonResponse>("reason", {
    userInput: userInput || null,
  });
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
