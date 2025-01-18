use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use std::io::BufRead;
use git2::{Repository, Commit, ObjectType, Time};
use chrono::{DateTime, Utc};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

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

        // First, count total commits
        let mut count_cmd = std::process::Command::new("git");
        count_cmd.current_dir(&repo_root)
            .arg("rev-list")
            .arg("--count")
            .arg("HEAD");

        if since != "all" {
            count_cmd.arg(format!("--since={}", since));
        }
        if let Some(ref filter) = self.path_filter {
            count_cmd.arg("--").arg(filter);
        }

        let total_commits = String::from_utf8_lossy(&count_cmd.output().expect("Failed to count commits").stdout)
            .trim()
            .parse::<u64>()
            .unwrap_or(0);

        // Setup progress bar
        let progress_bar = ProgressBar::new(total_commits);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} commits ({per_sec})")
                .unwrap()
                .progress_chars("#>-")
        );
        progress_bar.enable_steady_tick(Duration::from_millis(100));

        // Build git log command with numstat to get file changes
        let mut cmd = std::process::Command::new("git");
        cmd.current_dir(&repo_root)
            .arg("log")
            .arg("--no-merges")
            .arg("--format=%H%n%at%n%aN%x00")
            .arg("--numstat")
            .arg("--no-renames")
            .stdout(std::process::Stdio::piped());

        if since != "all" {
            cmd.arg(format!("--since={}", since));
        }
        if let Some(ref filter) = self.path_filter {
            cmd.arg("--").arg(filter);
        }

        let mut child = cmd.spawn().expect("Failed to spawn git command");
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let reader = std::io::BufReader::new(stdout);
        let mut lines = reader.lines().peekable();

        let mut commit_count = 0;
        let mut current_hash = String::new();
        let mut current_time = 0;
        let mut current_author = String::new();

        while let Some(line_result) = lines.next() {
            let line = line_result.expect("Failed to read line");
            
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            if line.len() == 40 { // Git hash
                commit_count += 1;
                progress_bar.set_position(commit_count);
                current_hash = line;
                
                if let Some(Ok(timestamp)) = lines.next() {
                    current_time = timestamp.parse().unwrap_or(0);
                }
                if let Some(Ok(author)) = lines.next() {
                    current_author = author;
                }
                continue;
            }

            // Parse stat line
            if let Some((file_path, _, _)) = parse_stat_line(&line) {
                let commit_time = DateTime::<Utc>::from_timestamp(current_time, 0)
                    .expect("Invalid timestamp");
                process_file_change(&mut hotspots, &existing_files, file_path, commit_time, &current_author);
            }
        }

        progress_bar.finish_with_message("Analysis complete");

        let mut result: Vec<FileHotspot> = hotspots.into_values().collect();
        result.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));
        
        eprintln!("\nFound {} files with changes", result.len());
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
    let mut output = String::from("High Churn Files:\n\n".bold().to_string());

    for (i, hotspot) in hotspots.iter().enumerate().take(10) {
        // File path with index
        output.push_str(&format!(
            "{}. {}\n",
            (i + 1).to_string().blue(),
            hotspot.path.green()
        ));

        // Commit info with count in yellow
        let commit_info = if since == "all" {
            format!("   - Commits: {}\n", hotspot.commit_count.to_string().yellow())
        } else {
            format!("   - Commits: {} since {}\n", 
                hotspot.commit_count.to_string().yellow(),
                since.blue()
            )
        };
        output.push_str(&commit_info);

        // Contributors count in cyan
        output.push_str(&format!(
            "   - Contributors: {}\n", 
            hotspot.contributor_count.to_string().cyan()
        ));
        
        // Suggestions in different colors based on type
        if hotspot.commit_count > 20 && hotspot.contributor_count > 4 {
            output.push_str(&"   - Suggestion: ".dimmed().to_string());
            output.push_str(&"Consider refactoring or adding more tests\n".red().to_string());
        } else if hotspot.contributor_count > 6 {
            output.push_str(&"   - Suggestion: ".dimmed().to_string());
            output.push_str(&"Consider assigning a code owner\n".yellow().to_string());
        } else if hotspot.commit_count > 15 {
            output.push_str(&"   - Suggestion: ".dimmed().to_string());
            output.push_str(&"Review for potential technical debt\n".magenta().to_string());
        }
        output.push('\n');
    }

    output
} 