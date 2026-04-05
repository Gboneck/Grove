//! Qdrant vector memory — semantic search for long-term memory.
//!
//! Uses Qdrant's HTTP API directly (no SDK dependency).
//! Embeddings are generated using a simple TF-IDF-like bag-of-words approach
//! locally, with optional upgrade to model-based embeddings in the future.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

const QDRANT_URL: &str = "http://localhost:6333";
const COLLECTION_NAME: &str = "grove_memory";
const VECTOR_DIM: usize = 128;

/// A memory point stored in Qdrant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoint {
    pub id: String,
    pub content: String,
    pub category: String,
    pub confidence: f64,
    pub created_at: String,
    pub metadata: HashMap<String, String>,
}

/// Check if Qdrant is available.
pub async fn is_available() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();
    client
        .get(format!("{}/collections", QDRANT_URL))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Ensure the collection exists with the right schema.
pub async fn ensure_collection() -> Result<(), String> {
    let client = reqwest::Client::new();

    // Check if collection exists
    let resp = client
        .get(format!("{}/collections/{}", QDRANT_URL, COLLECTION_NAME))
        .send()
        .await
        .map_err(|e| format!("Qdrant connection failed: {}", e))?;

    if resp.status().is_success() {
        return Ok(());
    }

    // Create collection
    let body = json!({
        "vectors": {
            "size": VECTOR_DIM,
            "distance": "Cosine"
        }
    });

    let resp = client
        .put(format!("{}/collections/{}", QDRANT_URL, COLLECTION_NAME))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to create collection: {}", e))?;

    if resp.status().is_success() {
        eprintln!("[grove:qdrant] Created collection '{}'", COLLECTION_NAME);
        Ok(())
    } else {
        let text = resp.text().await.unwrap_or_default();
        Err(format!("Failed to create collection: {}", text))
    }
}

/// Store a memory point in Qdrant.
pub async fn upsert(point: &MemoryPoint) -> Result<(), String> {
    let client = reqwest::Client::new();
    let vector = embed_text(&point.content);

    let payload = json!({
        "content": point.content,
        "category": point.category,
        "confidence": point.confidence,
        "created_at": point.created_at,
        "metadata": point.metadata,
    });

    // Use a hash of the ID as numeric point ID
    let numeric_id = hash_to_u64(&point.id);

    let body = json!({
        "points": [{
            "id": numeric_id,
            "vector": vector,
            "payload": payload,
        }]
    });

    let resp = client
        .put(format!(
            "{}/collections/{}/points",
            QDRANT_URL, COLLECTION_NAME
        ))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Qdrant upsert failed: {}", e))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let text = resp.text().await.unwrap_or_default();
        Err(format!("Qdrant upsert error: {}", text))
    }
}

/// Semantic search — find similar memories.
pub async fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
    let client = reqwest::Client::new();
    let vector = embed_text(query);

    let body = json!({
        "vector": vector,
        "limit": limit,
        "with_payload": true,
        "score_threshold": 0.3,
    });

    let resp = client
        .post(format!(
            "{}/collections/{}/points/search",
            QDRANT_URL, COLLECTION_NAME
        ))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Qdrant search failed: {}", e))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Qdrant search error: {}", text));
    }

    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse search response: {}", e))?;

    let results = data
        .get("result")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let score = item.get("score")?.as_f64()?;
                    let payload = item.get("payload")?;
                    Some(SearchResult {
                        content: payload
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        category: payload
                            .get("category")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        confidence: payload
                            .get("confidence")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                        score,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(results)
}

/// Delete all points in the collection (for testing/reset).
pub async fn clear() -> Result<(), String> {
    let client = reqwest::Client::new();
    client
        .delete(format!("{}/collections/{}", QDRANT_URL, COLLECTION_NAME))
        .send()
        .await
        .map_err(|e| format!("Qdrant clear failed: {}", e))?;
    ensure_collection().await
}

/// Sync JSON-based long-term entries to Qdrant.
pub async fn sync_from_json() -> Result<usize, String> {
    let entries = super::longterm::read_entries();
    if entries.is_empty() {
        return Ok(0);
    }

    ensure_collection().await?;

    let mut count = 0;
    for entry in &entries {
        let point = MemoryPoint {
            id: entry.id.clone(),
            content: entry.content.clone(),
            category: format!("{:?}", entry.category).to_lowercase(),
            confidence: entry.confidence,
            created_at: entry.first_observed.clone(),
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "confirmations".to_string(),
                    entry.confirmation_count.to_string(),
                );
                m.insert(
                    "last_confirmed".to_string(),
                    entry.last_confirmed.clone(),
                );
                m
            },
        };
        if upsert(&point).await.is_ok() {
            count += 1;
        }
    }

    eprintln!("[grove:qdrant] Synced {} entries to Qdrant", count);
    Ok(count)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub content: String,
    pub category: String,
    pub confidence: f64,
    pub score: f64,
}

/// Synchronous search — tries Qdrant first, falls back to offline cosine search.
/// Used by context gathering which runs in sync Tauri command context.
pub fn search_sync(query: &str, limit: usize) -> Option<Vec<SearchResult>> {
    let vector = embed_text(query);
    let body = serde_json::json!({
        "vector": vector,
        "limit": limit,
        "with_payload": true,
        "score_threshold": 0.3,
    });
    let url = format!(
        "{}/collections/{}/points/search",
        QDRANT_URL, COLLECTION_NAME
    );

    // Run blocking HTTP on a separate OS thread to avoid panicking tokio
    // when reqwest::blocking drops its internal runtime in an async context.
    let body_str = body.to_string();
    let qdrant_result = std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(1))
            .build()
            .ok()?;
        let resp = client
            .post(&url)
            .header("content-type", "application/json")
            .body(body_str)
            .send()
            .ok()?;
        if resp.status().is_success() {
            resp.json::<serde_json::Value>().ok()
        } else {
            None
        }
    }).join().ok().flatten();

    if let Some(data) = qdrant_result {
        let results = parse_search_results(&data);
        if !results.is_empty() {
            return Some(results);
        }
    }

    // Fallback: offline cosine similarity search on JSON long-term entries
    Some(offline_search(query, limit))
}

/// Offline cosine similarity search against long-term JSON entries.
/// No Qdrant needed — computes embeddings locally and compares.
fn offline_search(query: &str, limit: usize) -> Vec<SearchResult> {
    let entries = super::longterm::read_entries();
    if entries.is_empty() {
        return Vec::new();
    }

    let query_vec = embed_text(query);
    let mut scored: Vec<(SearchResult, f64)> = entries
        .iter()
        .map(|entry| {
            let entry_vec = embed_text(&entry.content);
            let score: f64 = query_vec
                .iter()
                .zip(entry_vec.iter())
                .map(|(a, b)| a * b)
                .sum();
            (
                SearchResult {
                    content: entry.content.clone(),
                    category: format!("{:?}", entry.category).to_lowercase(),
                    confidence: entry.confidence,
                    score,
                },
                score,
            )
        })
        .filter(|(_, score)| *score > 0.2)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored.into_iter().map(|(r, _)| r).collect()
}

/// Parse Qdrant search response into SearchResult vec.
fn parse_search_results(data: &serde_json::Value) -> Vec<SearchResult> {
    data.get("result")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let score = item.get("score")?.as_f64()?;
                    let payload = item.get("payload")?;
                    Some(SearchResult {
                        content: payload.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        category: payload.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        confidence: payload.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        score,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

// --- Local embedding (bag-of-words TF-IDF approximation) ---

/// Generate a fixed-size vector from text using character n-gram hashing.
/// This is a lightweight local approach — can be upgraded to model embeddings later.
fn embed_text(text: &str) -> Vec<f64> {
    let lower = text.to_lowercase();
    let mut vector = vec![0.0f64; VECTOR_DIM];

    // Character trigram hashing
    let chars: Vec<char> = lower.chars().collect();
    for window in chars.windows(3) {
        let hash = simple_hash(&window.iter().collect::<String>());
        let idx = (hash as usize) % VECTOR_DIM;
        vector[idx] += 1.0;
    }

    // Word-level hashing for broader semantics
    for word in lower.split_whitespace() {
        if word.len() > 2 {
            let hash = simple_hash(word);
            let idx = (hash as usize) % VECTOR_DIM;
            vector[idx] += 2.0; // Words weighted more than character n-grams
        }
    }

    // L2 normalize
    let magnitude: f64 = vector.iter().map(|v| v * v).sum::<f64>().sqrt();
    if magnitude > 0.0 {
        for v in &mut vector {
            *v /= magnitude;
        }
    }

    vector
}

/// Simple FNV-1a inspired hash for strings.
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Hash a string ID to a u64 for Qdrant point IDs.
fn hash_to_u64(s: &str) -> u64 {
    simple_hash(s) & 0x7FFFFFFFFFFFFFFF // Ensure positive
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_produces_fixed_size() {
        let v = embed_text("hello world");
        assert_eq!(v.len(), VECTOR_DIM);
    }

    #[test]
    fn test_embed_normalized() {
        let v = embed_text("test embedding normalization");
        let mag: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((mag - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_similar_texts_closer() {
        let v1 = embed_text("user prefers morning work sessions");
        let v2 = embed_text("user likes working in the morning");
        let v3 = embed_text("rust programming language features");

        let sim_12: f64 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let sim_13: f64 = v1.iter().zip(v3.iter()).map(|(a, b)| a * b).sum();

        assert!(
            sim_12 > sim_13,
            "Similar texts should have higher cosine similarity"
        );
    }

    #[test]
    fn test_hash_to_u64_consistent() {
        let id = "test-uuid-12345";
        assert_eq!(hash_to_u64(id), hash_to_u64(id));
    }

    #[test]
    fn test_embed_empty_string() {
        let v = embed_text("");
        // Should not panic, all zeros is fine for empty
        assert_eq!(v.len(), VECTOR_DIM);
    }
}
