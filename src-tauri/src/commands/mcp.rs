use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::memory;
use crate::memory::working;
use crate::memory::longterm;
use crate::models::context::GroveContext;
use crate::soul::parser::Soul;
use crate::soul::evolution::RelationshipPhase;

/// MCP tool definitions that Grove exposes
#[derive(Debug, Clone, Serialize)]
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP server state
pub struct McpState(pub Arc<Mutex<McpServer>>);

pub struct McpServer {
    pub enabled: bool,
    pub port: u16,
}

impl McpServer {
    pub fn new() -> Self {
        McpServer {
            enabled: false,
            port: 0,
        }
    }
}

fn grove_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "grove_get_context".to_string(),
            description: "Get the current Grove OS context including soul, ventures, and memory summary".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDef {
            name: "grove_get_soul".to_string(),
            description: "Get the user's Soul.md identity document".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDef {
            name: "grove_get_memory".to_string(),
            description: "Get recent sessions, facts, and patterns from Grove's memory".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "sessions_count": { "type": "number", "description": "Number of recent sessions to return (default 5)" }
                },
                "required": []
            }),
        },
        McpToolDef {
            name: "grove_get_facts".to_string(),
            description: "Get semantic facts Grove has learned about the user".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "category": { "type": "string", "description": "Filter by category: identity, preference, goal, skill, relationship" }
                },
                "required": []
            }),
        },
        McpToolDef {
            name: "grove_add_fact".to_string(),
            description: "Add a new fact about the user to Grove's memory".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "category": { "type": "string", "description": "Fact category: identity, preference, goal, skill, relationship" },
                    "content": { "type": "string", "description": "The fact content" }
                },
                "required": ["category", "content"]
            }),
        },
        McpToolDef {
            name: "grove_get_ventures".to_string(),
            description: "Get the user's active ventures/projects from context.json".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDef {
            name: "grove_get_priority".to_string(),
            description: "Get the user's current top priority based on ventures, deadlines, and recent activity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDef {
            name: "grove_what_changed".to_string(),
            description: "Get a summary of what changed since the last session — new facts, patterns, venture updates".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "max_entries": { "type": "number", "description": "Max number of recent changes to return (default 10)" }
                },
                "required": []
            }),
        },
        McpToolDef {
            name: "grove_get_focus".to_string(),
            description: "Get the user's current focus state: relationship phase, active role, soul completeness, and recommended next actions".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

/// Handle an MCP tool call
pub fn handle_tool_call(name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "grove_get_context" => {
            let context = GroveContext::gather(None)
                .map_err(|e| format!("Failed to gather context: {}", e))?;
            Ok(json!({
                "local_time": context.local_time,
                "day_of_week": context.day_of_week,
                "date": context.date,
                "last_seen": context.last_seen,
                "recent_memory": context.recent_memory,
                "accumulated_insights": context.accumulated_insights,
            }))
        }
        "grove_get_soul" => {
            let grove_dir = dirs::home_dir()
                .ok_or("Could not find home directory")?
                .join(".grove");
            let soul = std::fs::read_to_string(grove_dir.join("soul.md"))
                .map_err(|e| format!("Failed to read soul.md: {}", e))?;
            Ok(json!({ "soul_md": soul }))
        }
        "grove_get_memory" => {
            let count = args
                .get("sessions_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(5) as usize;
            let mem = memory::read_memory_file()?;
            let sessions: Vec<_> = mem.sessions.iter().rev().take(count).collect();
            Ok(json!({
                "sessions": sessions,
                "total_sessions": mem.tuning.total_sessions,
                "facts_count": mem.facts.len(),
                "patterns_count": mem.patterns.len(),
            }))
        }
        "grove_get_facts" => {
            let mem = memory::read_memory_file()?;
            let category_filter = args
                .get("category")
                .and_then(|v| v.as_str());
            let facts: Vec<_> = mem.facts.iter()
                .filter(|f| f.superseded_by.is_none() && f.confidence >= 0.3)
                .filter(|f| category_filter.is_none() || Some(f.category.as_str()) == category_filter)
                .collect();
            Ok(json!({ "facts": facts }))
        }
        "grove_add_fact" => {
            let category = args
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'category' argument")?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content' argument")?;
            memory::upsert_fact(category, content, "mcp_external")?;
            Ok(json!({ "success": true, "message": "Fact added to memory" }))
        }
        "grove_get_ventures" => {
            let grove_dir = dirs::home_dir()
                .ok_or("Could not find home directory")?
                .join(".grove");
            let context_str = std::fs::read_to_string(grove_dir.join("context.json"))
                .map_err(|e| format!("Failed to read context.json: {}", e))?;
            let context: Value = serde_json::from_str(&context_str)
                .map_err(|e| format!("Failed to parse context.json: {}", e))?;
            Ok(context)
        }
        "grove_get_priority" => {
            let grove_dir = dirs::home_dir()
                .ok_or("Could not find home directory")?
                .join(".grove");
            let context_str = std::fs::read_to_string(grove_dir.join("context.json"))
                .map_err(|e| format!("Failed to read context.json: {}", e))?;
            let context: Value = serde_json::from_str(&context_str)
                .map_err(|e| format!("Failed to parse context.json: {}", e))?;

            // Find highest priority venture
            let ventures = context.get("ventures")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let top_venture = ventures.iter()
                .filter(|v| v.get("status").and_then(|s| s.as_str()) != Some("completed"))
                .min_by_key(|v| v.get("priority").and_then(|p| p.as_u64()).unwrap_or(999))
                .cloned();

            // Get recent memory for context
            let recent = working::recent_entries(500);

            Ok(json!({
                "top_venture": top_venture,
                "active_ventures_count": ventures.iter()
                    .filter(|v| v.get("status").and_then(|s| s.as_str()) != Some("completed"))
                    .count(),
                "recent_activity_snippet": recent.lines().take(5).collect::<Vec<_>>().join("\n"),
            }))
        }
        "grove_what_changed" => {
            let max_entries = args
                .get("max_entries")
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as usize;

            // Recent working memory entries
            let recent = working::recent_entries(3000);
            let entries: Vec<&str> = recent.split("### ")
                .filter(|s| !s.trim().is_empty())
                .take(max_entries)
                .collect();

            // Recent long-term pattern promotions
            let lt_summary = longterm::context_summary();

            // Recent facts
            let mem = memory::read_memory_file().unwrap_or_default();
            let recent_facts: Vec<_> = mem.facts.iter()
                .filter(|f| f.superseded_by.is_none())
                .rev()
                .take(5)
                .map(|f| json!({
                    "category": f.category,
                    "content": f.content,
                    "confidence": f.confidence,
                }))
                .collect();

            Ok(json!({
                "recent_journal_entries": entries,
                "longterm_patterns": lt_summary,
                "recent_facts": recent_facts,
                "total_sessions": mem.tuning.total_sessions,
            }))
        }
        "grove_get_focus" => {
            let grove_dir = dirs::home_dir()
                .ok_or("Could not find home directory")?
                .join(".grove");
            let soul_raw = std::fs::read_to_string(grove_dir.join("soul.md"))
                .unwrap_or_default();
            let soul = Soul::parse(&soul_raw);
            let mem = memory::read_memory_file().unwrap_or_default();
            let phase = RelationshipPhase::from_metrics(
                soul.completeness(),
                mem.sessions.len() as u32,
            );
            let weak = soul.weak_sections(0.5);
            let weak_names: Vec<&str> = weak.iter().map(|s| s.heading.as_str()).collect();

            Ok(json!({
                "soul_completeness": format!("{:.0}%", soul.completeness() * 100.0),
                "relationship_phase": phase.display_name(),
                "autonomy_level": phase.autonomy_level(),
                "soul_gaps": weak_names,
                "total_sessions": mem.sessions.len(),
                "total_facts": mem.facts.len(),
                "total_patterns": mem.patterns.len(),
            }))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

/// Tauri command: list available MCP tools
#[tauri::command]
pub async fn mcp_list_tools() -> Result<Vec<McpToolDef>, String> {
    Ok(grove_tools())
}

/// Tauri command: call an MCP tool
#[tauri::command]
pub async fn mcp_call_tool(
    name: String,
    arguments: Option<Value>,
) -> Result<Value, String> {
    let args = arguments.unwrap_or(json!({}));
    handle_tool_call(&name, &args)
}

/// MCP JSON-RPC request handler for stdin/stdout mode
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

/// Handle a single JSON-RPC MCP request
pub fn handle_jsonrpc(request: &JsonRpcRequest) -> JsonRpcResponse {
    let result = match request.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "grove-os",
                "version": "0.5.0"
            }
        })),
        "tools/list" => {
            let tools: Vec<Value> = grove_tools()
                .into_iter()
                .map(|t| json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema,
                }))
                .collect();
            Ok(json!({ "tools": tools }))
        }
        "tools/call" => {
            let tool_name = request.params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = request.params
                .get("arguments")
                .cloned()
                .unwrap_or(json!({}));
            match handle_tool_call(tool_name, &arguments) {
                Ok(result) => Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                    }]
                })),
                Err(e) => Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": e
                    }],
                    "isError": true
                })),
            }
        }
        "notifications/initialized" => {
            // Client acknowledgment, no response needed
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(json!({})),
                error: None,
            };
        }
        _ => Err(format!("Unknown method: {}", request.method)),
    };

    match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(value),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(json!({
                "code": -32603,
                "message": e
            })),
        },
    }
}
