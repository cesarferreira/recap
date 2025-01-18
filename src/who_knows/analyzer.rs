use std::collections::HashMap;
use std::process::Command;
use std::path::Path;
use colored::*;
use chrono::{DateTime, Local};
use crate::who_knows::types::ContributorStats;

pub fn analyze_file_expertise(path: &str) -> Result<Vec<ContributorStats>, String> {
    // Check if path exists
    if !Path::new(path).exists() {
        return Err(format!("Path '{}' does not exist", path.blue()));
    }

    // Check if path is within a git repository
    let git_root = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|_| "Not a git repository".red().to_string())?;

    if !git_root.status.success() {
        return Err("Not inside a git repository".red().to_string());
    }

    let git_log = Command::new("git")
        .args(&[
            "log",
            "--follow",
            "--format=%H%x09%an%x09%at",
            "--",
            path,
        ])
        .output()
        .map_err(|e| format!("{}: {}", "Failed to execute git command".red(), e))?;

    if !git_log.status.success() {
        return Err(format!(
            "Git command failed: {}",
            String::from_utf8_lossy(&git_log.stderr).red()
        ));
    }

    let log_output = String::from_utf8_lossy(&git_log.stdout);
    if log_output.trim().is_empty() {
        return Err(format!("No git history found for '{}'", path.blue()));
    }

    let mut contributors: HashMap<String, ContributorStats> = HashMap::new();

    for line in log_output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 3 {
            continue;
        }

        let name = parts[1].to_string();
        let timestamp = parts[2]
            .parse::<i64>()
            .map_err(|_| "Failed to parse timestamp".red().to_string())?;
        
        let datetime = DateTime::from_timestamp(timestamp, 0)
            .ok_or("Invalid timestamp".red().to_string())?
            .with_timezone(&Local);

        if let Some(stats) = contributors.get_mut(&name) {
            stats.update(datetime);
        } else {
            contributors.insert(name.clone(), ContributorStats::new(name, datetime));
        }
    }

    let mut stats: Vec<ContributorStats> = contributors.into_values().collect();
    stats.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

    Ok(stats)
} 