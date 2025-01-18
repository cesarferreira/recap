use colored::*;
use regex::Regex;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::{Command, Stdio};

pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub relative_time: String,
    pub author: String,
}

pub struct GitStats {
    pub commits_count: i32,
    pub total_additions: i32,
    pub total_deletions: i32,
}

pub fn validate_repo(repo_path: &str) -> Result<(), String> {
    if !Path::new(repo_path).is_dir() {
        return Err(format!("Error: '{repo_path}' is not a valid directory."));
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map_err(|e| format!("Error running `git rev-parse`: {e}"))?;

    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != "true" {
        return Err(format!("Error: '{repo_path}' is not a Git repository."));
    }

    Ok(())
}

pub fn get_commits(repo_path: &str, author: &str, since: &str, show_diff: bool) -> Vec<GitCommit> {
    let format = if show_diff {
        "%h - %s [%cr by %an]%n"
    } else {
        "%h - %s [%cr by %an]"
    };

    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("--no-pager")
        .arg("log")
        .arg("--all")
        .arg(format!("--author={}", author))
        .arg(format!("--since={}", since))
        .arg(format!("--pretty=format:{}", format))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run git log");

    let re = Regex::new(r"^([0-9a-f]+) - (.*?) \[(.*?) by (.*?)\]$").unwrap();
    let mut commits = Vec::new();

    if let Some(stdout) = child.stdout.take() {
        let reader = io::BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line_str) = line {
                if let Some(caps) = re.captures(&line_str) {
                    commits.push(GitCommit {
                        hash: caps.get(1).unwrap().as_str().to_string(),
                        message: caps.get(2).unwrap().as_str().to_string(),
                        relative_time: caps.get(3).unwrap().as_str().to_string(),
                        author: caps.get(4).unwrap().as_str().to_string(),
                    });
                }
            }
        }
    }

    commits
}

pub fn get_commit_diff(repo_path: &str, commit_hash: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("--no-pager")
        .arg("show")
        .arg("--color=always")
        .arg("--stat")
        .arg("--patch")
        .arg(commit_hash)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

pub fn get_stats(repo_path: &str, author: &str, since: &str) -> GitStats {
    let mut stats = GitStats {
        commits_count: 0,
        total_additions: 0,
        total_deletions: 0,
    };

    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("log")
        .arg("--all")
        .arg(format!("--author={}", author))
        .arg(format!("--since={}", since))
        .arg("--pretty=tformat:COMMIT")
        .arg("--numstat")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run git log for stats");

    if let Some(stdout) = child.stdout.take() {
        let reader = io::BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let line_str = line.trim().to_string();
            if line_str == "COMMIT" {
                stats.commits_count += 1;
            } else if !line_str.is_empty() {
                let parts: Vec<&str> = line_str.split_whitespace().collect();
                if parts.len() >= 3 {
                    let added = parts[0].parse::<i32>().unwrap_or(0);
                    let deleted = parts[1].parse::<i32>().unwrap_or(0);
                    stats.total_additions += added;
                    stats.total_deletions += deleted;
                }
            }
        }
    }

    stats
} 