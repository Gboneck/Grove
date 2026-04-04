import { readFileSync } from "fs";
import { join } from "path";

export function getSoulMd(): string {
  const filePath = join(process.cwd(), "soul.md");
  return readFileSync(filePath, "utf-8");
}
