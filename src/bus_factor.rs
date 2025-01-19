use std::collections::HashMap;
use std::path::Path;
use colored::*;
use git2::{Repository, BlameOptions};
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
        
        // Get the absolute repository path
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
                    Err(_) => continue, // Skip files that can't be analyzed
                }
            } else if path.is_dir() && !path.ends_with(".git") {
                let _ = self.analyze_directory(&path, results);
            }
        }

        Ok(())
    }

    fn analyze_file(&self, file_path: &Path) -> Result<BusFactorResult, Box<dyn Error>> {
        let mut blame_opts = BlameOptions::new();
        blame_opts.track_copies_same_file(true);
        blame_opts.track_copies_same_commit_moves(true);

        let repo_path = self.repo.workdir()
            .ok_or("Could not get repository working directory")?;
        let relative_path = file_path.strip_prefix(repo_path)?;
        
        // Skip empty files
        if std::fs::read_to_string(file_path)?.trim().is_empty() {
            return Err("Empty file".into());
        }

        let blame = self.repo.blame_file(relative_path, Some(&mut blame_opts))?;
        let mut author_lines: HashMap<String, usize> = HashMap::new();
        let total_lines = blame.len();

        if total_lines == 0 {
            return Err("No lines to analyze".into());
        }

        for i in 0..total_lines {
            if let Some(hunk) = blame.get_line(i) {
                let signature = hunk.final_signature();
                let author = signature.name().unwrap_or("Unknown").to_string();
                *author_lines.entry(author).or_insert(0) += 1;
            }
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
    sorted_results.sort_by(|a, b| b.ownership_percentage.partial_cmp(&a.ownership_percentage).unwrap());

    for result in sorted_results {
        output.push_str(&format!(
            "  - {} ({}% owned by {}, {} lines)\n",
            result.path.blue(),
            format!("{:.1}", result.ownership_percentage).red(),
            result.dominant_author.green(),
            result.total_lines
        ));
    }

    output
} 