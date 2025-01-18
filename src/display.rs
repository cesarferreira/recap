use colored::*;
use tabled::{
    Table, Tabled,
    Style, Disable,
    Modify,
    object::Segment,
    Alignment
};
use crate::git::{GitCommit, GitStats};

#[derive(Tabled)]
struct StatsRow {
    label: String,
    value: String,
}

pub fn print_commit(commit: &GitCommit) {
    println!(
        "{} - {} [{} by {}]",
        commit.hash.yellow().bold(),
        commit.message.cyan(),
        commit.relative_time.green(),
        commit.author.magenta()
    );
}

pub fn print_diff(diff: &str) {
    for line in diff.lines() {
        println!("    {}", line);
    }
    println!();
}

pub fn print_stats(stats: &GitStats) {
    let stats_data = vec![
        StatsRow {
            label: "Commits".bold().to_string(),
            value: stats.commits_count.to_string().yellow().bold().to_string(),
        },
        StatsRow {
            label: "Lines added (+)".bold().to_string(),
            value: stats.total_additions.to_string().green().bold().to_string(),
        },
        StatsRow {
            label: "Lines deleted (-)".bold().to_string(),
            value: stats.total_deletions.to_string().red().bold().to_string(),
        },
    ];

    let table = Table::new(stats_data)
        .with(Style::modern())
        .with(Disable::Row(..1))
        .with(Modify::new(Segment::all()).with(Alignment::left()));

    println!();
    println!("{}", "====================== STATS ======================".bold());
    println!();
    println!("{table}");
} 