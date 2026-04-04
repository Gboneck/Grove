//! Grove OS MCP Server (stdio mode) + HTTP API mode
//!
//! Run as a standalone MCP server so external tools
//! (Claude Code, AI agents, etc.) can query Grove's context and memory.
//!
//! Usage:
//!   grove-mcp                    # stdio MCP mode (for Claude Code)
//!   grove-mcp --http 8377        # HTTP API mode (headless integrations)
//!
//! MCP config for Claude Code:
//! ```json
//! {
//!   "mcpServers": {
//!     "grove": { "command": "grove-mcp", "args": [] }
//!   }
//! }
//! ```

use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn grove_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

fn handle_tool_call(name: &str, args: &Value) -> Result<Value, String> {
    let dir = grove_dir();

    match name {
        "grove_get_soul" => {
            let soul = std::fs::read_to_string(dir.join("soul.md"))
                .map_err(|e| format!("Failed to read soul.md: {}", e))?;
            Ok(json!({ "soul_md": soul }))
        }
        "grove_get_ventures" => {
            let content = std::fs::read_to_string(dir.join("context.json"))
                .map_err(|e| format!("Failed to read context.json: {}", e))?;
            let ctx: Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse: {}", e))?;
            Ok(ctx)
        }
        "grove_get_memory" => {
            let content = std::fs::read_to_string(dir.join("memory.json"))
                .map_err(|e| format!("Failed to read memory.json: {}", e))?;
            let mem: Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse: {}", e))?;
            let count = args.get("sessions_count").and_then(|v| v.as_u64()).unwrap_or(5);
            let sessions = mem
                .get("sessions")
                .and_then(|s| s.as_array())
                .map(|arr| arr.iter().rev().take(count as usize).cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            Ok(json!({
                "sessions": sessions,
                "facts_count": mem.get("facts").and_then(|f| f.as_array()).map(|a| a.len()).unwrap_or(0),
            }))
        }
        "grove_get_facts" => {
            let content = std::fs::read_to_string(dir.join("memory.json"))
                .map_err(|e| format!("Failed to read: {}", e))?;
            let mem: Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse: {}", e))?;
            let category = args.get("category").and_then(|v| v.as_str());
            let facts = mem
                .get("facts")
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter(|f| {
                            f.get("superseded_by").and_then(|s| s.as_str()).is_none()
                        })
                        .filter(|f| {
                            category.is_none()
                                || f.get("category").and_then(|c| c.as_str()) == category
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Ok(json!({ "facts": facts }))
        }
        "grove_add_fact" => {
            let cat = args
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or("Missing category")?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing content")?;
            let mem_path = dir.join("memory.json");
            let mem_str =
                std::fs::read_to_string(&mem_path).unwrap_or_else(|_| "{}".to_string());
            let mut mem: Value = serde_json::from_str(&mem_str).unwrap_or(json!({}));
            let new_fact = json!({
                "id": uuid::Uuid::new_v4().to_string(),
                "category": cat,
                "content": content,
                "confidence": 0.7,
                "source": "mcp_external",
                "created_at": chrono::Utc::now().to_rfc3339(),
                "last_confirmed": chrono::Utc::now().to_rfc3339(),
                "superseded_by": null
            });
            if let Some(facts) = mem.get_mut("facts").and_then(|f| f.as_array_mut()) {
                facts.push(new_fact);
            }
            std::fs::write(
                &mem_path,
                serde_json::to_string_pretty(&mem).unwrap_or_default(),
            )
            .ok();
            Ok(json!({ "success": true }))
        }
        "grove_get_context" => {
            let soul = std::fs::read_to_string(dir.join("soul.md")).unwrap_or_default();
            let context = std::fs::read_to_string(dir.join("context.json")).unwrap_or_default();
            let now = chrono::Local::now();
            Ok(json!({
                "local_time": now.to_rfc3339(),
                "day_of_week": now.format("%A").to_string(),
                "date": now.format("%B %-d, %Y").to_string(),
                "soul_summary": if soul.len() > 500 { &soul[..500] } else { &soul },
                "context": serde_json::from_str::<Value>(&context).unwrap_or(json!({})),
            }))
        }
        "grove_get_priority" => {
            let context =
                std::fs::read_to_string(dir.join("context.json")).unwrap_or_default();
            let ctx: Value = serde_json::from_str(&context).unwrap_or(json!({}));
            let ventures = ctx
                .get("ventures")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let top = ventures
                .iter()
                .filter(|v| v.get("status").and_then(|s| s.as_str()) != Some("completed"))
                .min_by_key(|v| v.get("priority").and_then(|p| p.as_u64()).unwrap_or(999))
                .cloned();
            Ok(json!({
                "top_venture": top,
                "active_count": ventures.iter()
                    .filter(|v| v.get("status").and_then(|s| s.as_str()) != Some("completed"))
                    .count(),
            }))
        }
        "grove_what_changed" => {
            let max = args
                .get("max_entries")
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as usize;
            let memory_md =
                std::fs::read_to_string(dir.join("MEMORY.md")).unwrap_or_default();
            let entries: Vec<&str> = memory_md
                .split("### ")
                .filter(|s| !s.trim().is_empty())
                .take(max)
                .collect();
            Ok(json!({ "recent_entries": entries }))
        }
        "grove_get_focus" => {
            let soul_raw = std::fs::read_to_string(dir.join("soul.md")).unwrap_or_default();
            let mem_raw =
                std::fs::read_to_string(dir.join("memory.json")).unwrap_or_default();
            let mem: Value = serde_json::from_str(&mem_raw).unwrap_or(json!({}));
            let session_count = mem
                .get("sessions")
                .and_then(|s| s.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let facts_count = mem
                .get("facts")
                .and_then(|f| f.as_array())
                .map(|a| a.len())
                .unwrap_or(0);

            // Simple completeness estimate from soul.md section count
            let section_count = soul_raw.matches("## ").count();
            let completeness = (section_count as f64 * 0.12).min(1.0);

            Ok(json!({
                "soul_completeness": format!("{:.0}%", completeness * 100.0),
                "sessions": session_count,
                "facts": facts_count,
                "soul_sections": section_count,
            }))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "grove_get_context",
                "description": "Get the current Grove OS context including time, soul summary, and ventures",
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            },
            {
                "name": "grove_get_soul",
                "description": "Get the user's Soul.md identity document",
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            },
            {
                "name": "grove_get_memory",
                "description": "Get recent sessions from Grove's memory",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sessions_count": { "type": "number", "description": "Number of recent sessions (default 5)" }
                    }
                }
            },
            {
                "name": "grove_get_facts",
                "description": "Get semantic facts Grove has learned about the user",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "category": { "type": "string", "description": "Filter: identity, preference, goal, skill, relationship" }
                    }
                }
            },
            {
                "name": "grove_add_fact",
                "description": "Add a new fact about the user to Grove's memory",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "category": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["category", "content"]
                }
            },
            {
                "name": "grove_get_ventures",
                "description": "Get the user's active ventures/projects",
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            },
            {
                "name": "grove_get_priority",
                "description": "Get the user's current top priority venture",
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            },
            {
                "name": "grove_what_changed",
                "description": "Get recent changes — journal entries, facts, patterns",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "max_entries": { "type": "number", "description": "Max entries (default 10)" }
                    }
                }
            },
            {
                "name": "grove_get_focus",
                "description": "Get current focus state: soul completeness, session count, fact count",
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            }
        ]
    })
}

fn handle_request(req: &JsonRpcRequest) -> JsonRpcResponse {
    let result = match req.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "grove-os", "version": "0.6.0" }
        })),
        "notifications/initialized" => Ok(json!({})),
        "tools/list" => Ok(tools_list()),
        "tools/call" => {
            let name = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = req.params.get("arguments").cloned().unwrap_or(json!({}));
            match handle_tool_call(name, &args) {
                Ok(r) => Ok(json!({
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&r).unwrap_or_default() }]
                })),
                Err(e) => Ok(json!({
                    "content": [{ "type": "text", "text": e }],
                    "isError": true
                })),
            }
        }
        _ => Err(format!("Unknown method: {}", req.method)),
    };

    match result {
        Ok(v) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(v),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: None,
            error: Some(json!({"code": -32603, "message": e})),
        },
    }
}

fn run_stdio() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let response = handle_request(&request);
        let response_str = serde_json::to_string(&response).unwrap_or_default();
        let mut out = stdout.lock();
        writeln!(out, "{}", response_str).ok();
        out.flush().ok();
    }
}

/// Minimal HTTP API server for headless mode.
/// Supports: GET /api/tool/:name and POST /api/tool/:name
#[tokio::main]
async fn run_http(port: u16) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    eprintln!("[grove-mcp] HTTP API listening on http://{}", addr);

    loop {
        let (mut socket, _) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => continue,
        };

        tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            let n = match socket.read(&mut buf).await {
                Ok(n) => n,
                Err(_) => return,
            };

            let request_str = String::from_utf8_lossy(&buf[..n]);
            let lines: Vec<&str> = request_str.lines().collect();

            if lines.is_empty() {
                return;
            }

            // Parse first line: "GET /api/tool/grove_get_soul HTTP/1.1"
            let parts: Vec<&str> = lines[0].split_whitespace().collect();
            if parts.len() < 2 {
                return;
            }

            let method = parts[0];
            let path = parts[1];

            // Parse body for POST
            let body = request_str
                .find("\r\n\r\n")
                .map(|i| &request_str[i + 4..])
                .unwrap_or("");
            let args: Value = serde_json::from_str(body).unwrap_or(json!({}));

            let response_body = if path.starts_with("/api/tool/") {
                let tool_name = &path[10..];
                match handle_tool_call(tool_name, &args) {
                    Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_default(),
                    Err(e) => json!({"error": e}).to_string(),
                }
            } else if path == "/api/tools" {
                serde_json::to_string_pretty(&tools_list()).unwrap_or_default()
            } else if path == "/health" {
                json!({"status": "ok", "version": "0.6.0"}).to_string()
            } else {
                json!({"error": "Not found. Use /api/tool/<name> or /api/tools"}).to_string()
            };

            let http_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                response_body.len(),
                response_body
            );

            socket.write_all(http_response.as_bytes()).await.ok();
        });
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 3 && args[1] == "--http" {
        let port: u16 = args[2].parse().unwrap_or(8377);
        run_http(port);
    } else {
        run_stdio();
    }
}
