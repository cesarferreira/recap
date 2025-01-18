use crate::who_knows::types::ContributorStats;

pub fn display_expertise(path: &str, stats: Vec<ContributorStats>) {
    println!("\nFile: {}\n", path);

    for (i, stat) in stats.iter().enumerate() {
        println!("{}. {}", i + 1, stat.name);
        println!("   - Changes: {}", stat.commit_count);
        println!("   - Last Touched: {}", stat.format_last_touched());
        println!(
            "   - Contribution Duration: {} – {} ({} total)\n",
            stat.first_commit.format("%b %Y"),
            stat.last_commit.format("%b %Y"),
            stat.contribution_duration()
        );
    }
} 