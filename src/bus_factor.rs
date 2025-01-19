use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use colored::*;
use git2::Repository;
use std::error::Error;

pub struct BusFactorAnalyzer {
    repo: Repository,
    threshold: f64,
}

#[derive(Debug, Clone)]
pub struct BusFactorResult {
    pub path: String,
    pub dominant_author: String,
    pub ownership_percentage: f64,
    pub total_lines: usize,
}

impl BusFactorAnalyzer {
    pub fn new(repo_path: &str, threshold: f64) -> Result<Self, Box<dyn Error>> {
        let repo = Repository::open(repo_path)?;
        Ok(BusFactorAnalyzer { repo, threshold })
    }

    pub fn analyze_path(&self, path: &str) -> Result<Vec<BusFactorResult>, Box<dyn Error>> {
        let mut results = Vec::new();
        let path = Path::new(path);
        
        let repo_path = self.repo.workdir()
            .ok_or("Could not get repository working directory")?;
        
        let target_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_path.join(path)
        };

        if target_path.is_file() {
            if let Ok(result) = self.analyze_file(&target_path) {
                if result.ownership_percentage >= self.threshold {
                    results.push(result);
                }
            }
        } else {
            self.analyze_directory(&target_path, &mut results)?;
        }

        Ok(results)
    }

    fn analyze_directory(&self, dir_path: &Path, results: &mut Vec<BusFactorResult>) -> Result<(), Box<dyn Error>> {
        let entries = std::fs::read_dir(dir_path)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Skip hidden files and .git directory
            if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false) {
                continue;
            }
            
            if path.is_file() {
                // Skip binary files and specific extensions
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ["exe", "dll", "so", "dylib", "png", "jpg", "jpeg", "gif", "pdf"]
                        .contains(&ext) {
                        continue;
                    }
                }
                
                match self.analyze_file(&path) {
                    Ok(result) => {
                        if result.ownership_percentage >= self.threshold {
                            results.push(result);
                        }
                    }
                    Err(_) => continue,
                }
            } else if path.is_dir() && !path.ends_with(".git") {
                let _ = self.analyze_directory(&path, results);
            }
        }

        Ok(())
    }

    fn analyze_file(&self, file_path: &Path) -> Result<BusFactorResult, Box<dyn Error>> {
        let repo_path = self.repo.workdir()
            .ok_or("Could not get repository working directory")?;
        let relative_path = file_path.strip_prefix(repo_path)?;
        
        // Skip empty files
        let content = std::fs::read_to_string(file_path)?;
        if content.trim().is_empty() {
            return Err("Empty file".into());
        }

        // Run git blame command
        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("blame")
            .arg("--line-porcelain") // Get detailed info including author name
            .arg(relative_path)
            .output()?;

        if !output.status.success() {
            return Err("Failed to run git blame".into());
        }

        let blame_output = String::from_utf8(output.stdout)?;
        let mut author_lines: HashMap<String, usize> = HashMap::new();
        let mut current_author = String::new();
        let mut total_lines = 0;
        let mut in_multiline_comment = false;

        for line in blame_output.lines() {
            if line.starts_with("author ") {
                current_author = line[7..].to_string();
            } else if line.starts_with('\t') {
                let code_line = line[1..].trim();
                
                // Skip empty lines
                if code_line.is_empty() {
                    continue;
                }

                // Handle multi-line comments
                if code_line.starts_with("/*") {
                    in_multiline_comment = true;
                    continue;
                }
                if code_line.ends_with("*/") {
                    in_multiline_comment = false;
                    continue;
                }
                if in_multiline_comment || code_line.starts_with("*") {
                    continue;
                }

                // Skip single-line comments
                if code_line.starts_with("//") {
                    continue;
                }

                total_lines += 1;
                *author_lines.entry(current_author.clone()).or_insert(0) += 1;
            }
        }

        if total_lines == 0 {
            return Err("No lines to analyze".into());
        }

        let (dominant_author, lines) = author_lines
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .unwrap_or(("Unknown".to_string(), 0));

        let ownership_percentage = (lines as f64 / total_lines as f64) * 100.0;

        Ok(BusFactorResult {
            path: relative_path.to_string_lossy().to_string(),
            dominant_author,
            ownership_percentage,
            total_lines,
        })
    }
}

pub fn format_bus_factor_report(results: &[BusFactorResult]) -> String {
    if results.is_empty() {
        return "No files found with high bus factor risk.".yellow().to_string();
    }

    let mut output = String::from("\nHigh Risk (Bus Factor 1):\n");

    // Sort results by ownership percentage in descending order
    let mut sorted_results = results.to_vec();
    sorted_results.sort_by(|a, b| {
        // First sort by ownership percentage
        let cmp = b.ownership_percentage.partial_cmp(&a.ownership_percentage).unwrap();
        if cmp == std::cmp::Ordering::Equal {
            // Then by number of lines (larger files first)
            b.total_lines.cmp(&a.total_lines)
        } else {
            cmp
        }
    });

    for result in sorted_results {
        output.push_str(&format!(
            "  - {} ({}% owned by {}, {} lines)\n",
            result.path.blue(),
            format!("{:.0}", result.ownership_percentage).red(),
            result.dominant_author.green(),
            result.total_lines
        ));
    }

    output
} 