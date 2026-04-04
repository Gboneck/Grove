//! Tauri commands for Qdrant vector memory.

use crate::memory::vector;

/// Check if Qdrant is available and reachable.
#[tauri::command]
pub async fn vector_status() -> Result<serde_json::Value, String> {
    let available = vector::is_available().await;
    Ok(serde_json::json!({
        "available": available,
        "url": "http://localhost:6333",
        "collection": "grove_memory",
    }))
}

/// Sync long-term JSON entries into Qdrant for semantic search.
#[tauri::command]
pub async fn vector_sync() -> Result<serde_json::Value, String> {
    let count = vector::sync_from_json().await?;
    Ok(serde_json::json!({
        "synced": count,
    }))
}

/// Semantic search across stored memories.
#[tauri::command]
pub async fn vector_search(
    query: String,
    limit: Option<usize>,
) -> Result<Vec<vector::SearchResult>, String> {
    let query = crate::security::validate_user_input(&query)?;
    vector::search(&query, limit.unwrap_or(5)).await
}
