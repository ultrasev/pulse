use std::process::Command;
use std::path::PathBuf;
use serde::Serialize;
use tauri::command;

#[derive(Serialize, Debug)]
pub struct GitBranch {
    name: String,
    current: bool,
}

#[derive(Serialize, Debug)]
pub struct GitState {
    branches: Vec<GitBranch>,
    repo_path: String,
}

fn get_claude_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude")
}

#[command]
pub fn get_git_branches() -> Result<GitState, String> {
    let repo_path = get_claude_path();

    // Check if directory exists
    if !repo_path.exists() {
        return Err(format!("Repository path does not exist: {:?}", repo_path));
    }

    let output = Command::new("git")
        .arg("branch")
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Git command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (name, current) = if trimmed.starts_with('*') {
            (trimmed[1..].trim().to_string(), true)
        } else {
            (trimmed.to_string(), false)
        };

        branches.push(GitBranch { name, current });
    }

    Ok(GitState {
        branches,
        repo_path: repo_path.to_string_lossy().to_string(),
    })
}

#[command]
pub fn switch_git_branch(branch: String) -> Result<String, String> {
    let repo_path = get_claude_path();

    let output = Command::new("git")
        .arg("checkout")
        .arg(&branch)
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Failed to switch branch: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(format!("Switched to branch {}", branch))
}
