use std::collections::HashMap;
use std::path::Path;
use git2::{Repository, Commit, ObjectType, Time};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct FileHotspot {
    pub path: String,
    pub commit_count: usize,
    pub contributor_count: usize,
    pub last_modified: DateTime<Utc>,
    pub contributors: HashMap<String, usize>,
}

pub struct HotspotAnalyzer {
    repo: Repository,
    path_filter: Option<String>,
}

impl HotspotAnalyzer {
    pub fn new(repo_path: &str, path_filter: Option<String>) -> Result<Self, git2::Error> {
        let path = Path::new(repo_path);
        let repo = Repository::discover(path)?;
        Ok(Self { repo, path_filter })
    }

    pub fn analyze(&self, since: &str) -> Result<Vec<FileHotspot>, git2::Error> {
        let mut hotspots: HashMap<String, FileHotspot> = HashMap::new();
        
        // Get repository root path
        let repo_root = self.repo.workdir()
            .expect("Repository has no working directory")
            .to_string_lossy()
            .into_owned();

        eprintln!("Analyzing repository at: {}", repo_root);

        // Build git log command with numstat to get file changes
        let mut cmd = std::process::Command::new("git");
        cmd.current_dir(&repo_root)
            .arg("log")
            .arg("--no-merges")     // Exclude merge commits
            .arg("--format=%H%n%at%n%aN%x00") // Custom format with NUL separator
            .arg("--numstat")        // Get number of added/deleted lines
            .arg("--no-renames");    // Don't follow renames to keep it simple

        // Add since filter if specified
        if since != "all" {
            cmd.arg(format!("--since={}", since));
        }

        // Add path filter if specified
        if let Some(ref filter) = self.path_filter {
            cmd.arg("--").arg(filter);
        }

        let output = cmd.output().expect("Failed to execute git command");
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Get list of files that currently exist using git ls-files
        let mut existing_files = std::collections::HashSet::new();
        let mut ls_cmd = std::process::Command::new("git");
        ls_cmd.current_dir(&repo_root)
            .arg("ls-files");
        
        if let Some(ref filter) = self.path_filter {
            ls_cmd.arg(filter);
        }

        let ls_output = ls_cmd.output().expect("Failed to execute git ls-files");
        let ls_output_str = String::from_utf8_lossy(&ls_output.stdout);
        
        for file in ls_output_str.lines() {
            if !file.trim().is_empty() {
                existing_files.insert(file.to_string());
            }
        }

        eprintln!("Found {} files in current tree", existing_files.len());

        let mut commit_count = 0;
        let mut lines = output_str.lines().peekable();

        while let Some(hash) = lines.next() {
            // Skip empty lines
            if hash.trim().is_empty() {
                continue;
            }

            commit_count += 1;
            if commit_count % 100 == 0 {
                eprintln!("Processed {} commits...", commit_count);
            }

            // Parse commit metadata
            let timestamp = lines.next()
                .and_then(|t| t.parse::<i64>().ok())
                .unwrap_or(0);
            let author = lines.next().unwrap_or("unknown").to_string();
            let commit_time = DateTime::<Utc>::from_timestamp(timestamp, 0)
                .expect("Invalid timestamp");

            // Skip the NUL separator if present
            if let Some(line) = lines.next() {
                if !line.is_empty() {
                    // Parse as a stat line since it's not empty
                    if let Some((file_path, additions, deletions)) = parse_stat_line(line) {
                        process_file_change(&mut hotspots, &existing_files, file_path, commit_time, &author);
                    }
                }
            }

            // Process the stat lines until we hit an empty line or the next commit hash
            while let Some(line) = lines.peek() {
                if line.trim().is_empty() || line.len() == 40 { // Git hash is 40 chars
                    break;
                }
                if let Some((file_path, additions, deletions)) = parse_stat_line(lines.next().unwrap()) {
                    process_file_change(&mut hotspots, &existing_files, file_path, commit_time, &author);
                }
            }
        }

        let mut result: Vec<FileHotspot> = hotspots.into_values().collect();
        result.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));
        
        eprintln!("Analyzed {} commits", commit_count);
        eprintln!("Found {} files with changes", result.len());
        Ok(result)
    }
}

fn parse_stat_line(line: &str) -> Option<(&str, u32, u32)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 3 {
        return None;
    }

    let additions = parts[0].parse().unwrap_or(0);
    let deletions = parts[1].parse().unwrap_or(0);
    Some((parts[2], additions, deletions))
}

fn process_file_change(
    hotspots: &mut HashMap<String, FileHotspot>,
    existing_files: &std::collections::HashSet<String>,
    file_path: &str,
    commit_time: DateTime<Utc>,
    author: &str,
) {
    // Skip if file doesn't exist anymore
    if !existing_files.contains(file_path) {
        return;
    }

    let entry = hotspots.entry(file_path.to_string()).or_insert_with(|| FileHotspot {
        path: file_path.to_string(),
        commit_count: 0,
        contributor_count: 0,
        last_modified: commit_time,
        contributors: HashMap::new(),
    });

    entry.commit_count += 1;
    *entry.contributors.entry(author.to_string()).or_insert(0) += 1;
    entry.contributor_count = entry.contributors.len();
    
    if commit_time > entry.last_modified {
        entry.last_modified = commit_time;
    }
}

pub fn format_hotspot_report(hotspots: &[FileHotspot], since: &str) -> String {
    let mut output = String::from("High Churn Files:\n\n");

    for (i, hotspot) in hotspots.iter().enumerate().take(10) {
        output.push_str(&format!(
            "{}. {}\n",
            i + 1,
            hotspot.path
        ));
        let commit_info = if since == "all" {
            format!("   - Commits: {}\n", hotspot.commit_count)
        } else {
            format!("   - Commits: {} since {}\n", hotspot.commit_count, since)
        };
        output.push_str(&commit_info);
        output.push_str(&format!("   - Contributors: {}\n", hotspot.contributor_count));
        
        // Add suggestions based on metrics
        if hotspot.commit_count > 20 && hotspot.contributor_count > 4 {
            output.push_str("   - Suggestion: Consider refactoring or adding more tests\n");
        } else if hotspot.contributor_count > 6 {
            output.push_str("   - Suggestion: Consider assigning a code owner\n");
        } else if hotspot.commit_count > 15 {
            output.push_str("   - Suggestion: Review for potential technical debt\n");
        }
        output.push('\n');
    }

    output
} 