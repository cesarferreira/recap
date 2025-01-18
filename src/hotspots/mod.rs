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
        
        // Convert path_filter to absolute path if it exists
        let absolute_path_filter = path_filter.map(|p| {
            let filter_path = Path::new(&p);
            if filter_path.is_absolute() {
                p
            } else {
                path.join(filter_path)
                    .to_string_lossy()
                    .into_owned()
            }
        });

        Ok(Self { repo, path_filter: absolute_path_filter })
    }

    pub fn analyze(&self, since: &str) -> Result<Vec<FileHotspot>, git2::Error> {
        let mut hotspots: HashMap<String, FileHotspot> = HashMap::new();
        let head = self.repo.head()?;
        let obj = head.resolve()?.peel(ObjectType::Commit)?;
        let head_commit = obj.into_commit().expect("Could not find commit");

        // Get repository root path
        let repo_root = self.repo.workdir()
            .expect("Repository has no working directory")
            .to_string_lossy()
            .into_owned();

        // Parse the since parameter
        let since_time = if since == "all" {
            None
        } else {
            let output = std::process::Command::new("git")
                .arg("rev-parse")
                .arg(format!("--since={}", since))
                .output()
                .expect("Failed to execute git command");

            if output.status.success() {
                Some(DateTime::<Utc>::from_timestamp(
                    std::str::from_utf8(&output.stdout)
                        .unwrap()
                        .trim()
                        .parse()
                        .unwrap_or(0),
                    0,
                ).expect("Invalid timestamp"))
            } else {
                None
            }
        };

        self.traverse_commits(&head_commit, since_time, &mut hotspots, &repo_root)?;

        let mut result: Vec<FileHotspot> = hotspots.into_values().collect();
        result.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));
        Ok(result)
    }

    fn traverse_commits(
        &self,
        commit: &Commit,
        since_time: Option<DateTime<Utc>>,
        hotspots: &mut HashMap<String, FileHotspot>,
        repo_root: &str,
    ) -> Result<(), git2::Error> {
        let commit_time = DateTime::<Utc>::from_timestamp(commit.time().seconds(), 0)
            .expect("Invalid timestamp");

        if let Some(cutoff) = since_time {
            if commit_time < cutoff {
                return Ok(());
            }
        }

        // Process the commit's changes
        if let Ok(Some(parent)) = commit.parent(0).map(Some).or::<git2::Error>(Ok(None)) {
            let tree = commit.tree()?;
            let parent_tree = parent.tree()?;
            let diff = self.repo.diff_tree_to_tree(
                Some(&parent_tree),
                Some(&tree),
                None,
            )?;

            diff.foreach(
                &mut |delta, _| -> bool {
                    if let Some(path) = delta.new_file().path() {
                        let path_str = path.to_string_lossy().into_owned();
                        let absolute_path = Path::new(repo_root).join(&path_str);
                        let absolute_path_str = absolute_path.to_string_lossy().into_owned();
                        
                        // Apply path filter if specified
                        if let Some(ref filter) = self.path_filter {
                            if !absolute_path_str.starts_with(filter) {
                                return true;
                            }
                        }

                        let entry = hotspots.entry(path_str.clone()).or_insert_with(|| FileHotspot {
                            path: path_str,
                            commit_count: 0,
                            contributor_count: 0,
                            last_modified: commit_time,
                            contributors: HashMap::new(),
                        });

                        entry.commit_count += 1;
                        if let Some(author) = commit.author().name() {
                            *entry.contributors.entry(author.to_string()).or_insert(0) += 1;
                            entry.contributor_count = entry.contributors.len();
                        }
                    }
                    true
                },
                None,
                None,
                None,
            )?;
        }

        // Traverse parents
        for parent in commit.parents() {
            self.traverse_commits(&parent, since_time, hotspots, repo_root)?;
        }

        Ok(())
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