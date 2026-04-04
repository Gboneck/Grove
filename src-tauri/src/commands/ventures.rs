use serde_json::Value;
use std::fs;

use crate::models::VentureUpdate;

/// Apply model-suggested venture updates to context.json.
/// Returns descriptions of changes made.
pub fn apply_venture_updates(updates: &[VentureUpdate]) -> Vec<String> {
    let mut results = Vec::new();

    let grove_dir = match dirs::home_dir() {
        Some(h) => h.join(".grove"),
        None => return results,
    };

    let context_path = grove_dir.join("context.json");
    let content = match fs::read_to_string(&context_path) {
        Ok(c) => c,
        Err(_) => return results,
    };

    let mut context: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return results,
    };

    let ventures = match context.get_mut("ventures").and_then(|v| v.as_array_mut()) {
        Some(v) => v,
        None => return results,
    };

    for update in updates {
        // Find the venture by name
        let venture = ventures.iter_mut().find(|v| {
            v.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.to_lowercase() == update.venture_name.to_lowercase())
                .unwrap_or(false)
        });

        if let Some(v) = venture {
            if let Some(obj) = v.as_object_mut() {
                let old_value = obj
                    .get(&update.field)
                    .cloned()
                    .unwrap_or(Value::Null);

                obj.insert(update.field.clone(), update.new_value.clone());

                results.push(format!(
                    "Updated {}.{}: {} -> {} ({})",
                    update.venture_name,
                    update.field,
                    format_value(&old_value),
                    format_value(&update.new_value),
                    update.reason
                ));
            }
        } else {
            eprintln!(
                "[grove] Venture '{}' not found for update",
                update.venture_name
            );
        }
    }

    // Write back if any changes were made
    if !results.is_empty() {
        if let Ok(updated) = serde_json::to_string_pretty(&context) {
            fs::write(&context_path, updated).ok();
        }
    }

    results
}

fn format_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}
