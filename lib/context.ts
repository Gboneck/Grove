import { readFileSync } from "fs";
import { join } from "path";

export interface Venture {
  name: string;
  status: string;
  health: string;
  priority: number;
  nextAction: string;
  deadline?: string;
  notes: string;
}

export interface ContextData {
  ventures: Venture[];
}

export function getContext(): ContextData {
  const filePath = join(process.cwd(), "context.json");
  const raw = readFileSync(filePath, "utf-8");
  return JSON.parse(raw);
}
