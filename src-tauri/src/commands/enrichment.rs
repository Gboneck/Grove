//! Tauri command to route enrichment prompt answers directly into Soul.md.

use crate::soul::parser::Soul;
use crate::soul::patcher::SoulPatch;

/// Route a user's answer to an enrichment prompt directly into the appropriate
/// Soul.md section. This bypasses the full reasoning cycle for efficiency.
#[tauri::command]
pub async fn answer_enrichment(
    section: String,
    answer: String,
) -> Result<String, String> {
    let answer = crate::security::validate_user_input(&answer)?;

    let grove_dir = dirs::home_dir()
        .ok_or("No home directory")?
        .join(".grove");
    let soul_path = grove_dir.join("soul.md");

    let soul_raw = std::fs::read_to_string(&soul_path)
        .map_err(|e| format!("Failed to read soul.md: {}", e))?;
    let soul = Soul::parse(&soul_raw);

    let patch = SoulPatch {
        section: section.clone(),
        content: format!("- {}", answer),
        confidence_delta: 0.1,
        replace: false,
    };

    let evolved = soul.apply_patch(&patch);
    let markdown = evolved.to_markdown();
    std::fs::write(&soul_path, markdown)
        .map_err(|e| format!("Failed to write soul.md: {}", e))?;

    Ok(format!("Added to '{}' section", section))
}
