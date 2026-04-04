use std::fs;
use std::path::PathBuf;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

/// Generate a Soul.md from wizard answers
#[tauri::command]
pub async fn generate_soul(
    name: String,
    location: Option<String>,
    role: Option<String>,
    projects: Vec<String>,
    priorities: Vec<String>,
    work_style: Option<String>,
) -> Result<String, String> {
    let mut md = String::from("# Soul.md\n\n");

    // Identity section
    md.push_str("## Who I Am\n");
    md.push_str(&name);
    if let Some(ref loc) = location {
        if !loc.is_empty() {
            md.push_str(&format!(", based in {}", loc));
        }
    }
    md.push('\n');
    if let Some(ref r) = role {
        if !r.is_empty() {
            md.push_str(&format!("{}.\n", r));
        }
    }
    md.push('\n');

    // Projects section
    if !projects.is_empty() {
        md.push_str("## What I'm Working On\n");
        for project in &projects {
            if !project.is_empty() {
                md.push_str(&format!("- {}\n", project));
            }
        }
        md.push('\n');
    }

    // Priorities section
    if !priorities.is_empty() {
        md.push_str("## What Matters Right Now\n");
        for (i, priority) in priorities.iter().enumerate() {
            if !priority.is_empty() {
                md.push_str(&format!("{}. {}\n", i + 1, priority));
            }
        }
        md.push('\n');
    }

    // Work style section
    if let Some(ref style) = work_style {
        if !style.is_empty() {
            md.push_str("## How I Work\n");
            md.push_str(style);
            md.push_str("\n\n");
        }
    }

    // Write to disk
    let soul_path = grove_dir().join("soul.md");
    fs::write(&soul_path, &md).map_err(|e| format!("Failed to write soul.md: {}", e))?;

    Ok(md)
}

/// Check if the user has a personalized (non-default) Soul.md
#[tauri::command]
pub async fn is_soul_personalized() -> Result<bool, String> {
    let path = grove_dir().join("soul.md");
    if !path.exists() {
        return Ok(false);
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read soul.md: {}", e))?;

    Ok(!content.contains("[Your name, what you do, where you're based]"))
}
