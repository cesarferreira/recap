use colored::*;
use crate::who_knows::types::ContributorStats;

pub fn display_expertise(path: &str, stats: Vec<ContributorStats>) {
    println!("\n{}: {}\n", "File".bold(), path.blue());

    for (i, stat) in stats.iter().enumerate() {
        println!("{}. {}", (i + 1).to_string().yellow(), stat.name.green().bold());
        println!("   {} {}", "•".bright_black(), format!("Changes: {}", stat.commit_count).cyan());
        println!("   {} {}", "•".bright_black(), format!("Last Touched: {}", stat.format_last_touched()).magenta());
        println!(
            "   {} {}\n",
            "•".bright_black(),
            format!(
                "Contribution Duration: {} – {} ({})",
                stat.first_commit.format("%b %Y").to_string().yellow(),
                stat.last_commit.format("%b %Y").to_string().yellow(),
                stat.contribution_duration().bright_white()
            )
        );
    }
} 