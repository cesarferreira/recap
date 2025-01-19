use clap::{Arg, Command as ClapCommand, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Repository path
    #[arg(short, long, default_value = ".")]
    pub repo_path: String,

    /// Author name
    #[arg(short, long)]
    pub author: Option<String>,

    /// Show diff for each commit
    #[arg(short = 'd', long)]
    pub show_diff: bool,

    /// Play commit history as music
    #[arg(short = 'p', long)]
    pub play: bool,

    /// Save generated music to file
    #[arg(short = 's', long)]
    pub save_music_path: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze code hotspots
    Hotspots {
        /// Optional path to analyze (defaults to entire repository)
        path: Option<String>,
        
        /// Since date (e.g., "1 week ago", "2023-01-01")
        #[arg(short, long, default_value = "all")]
        since: String,
    },
    /// Show who knows about a specific file
    WhoKnows {
        /// Path to the file to analyze
        path: String,
    },
    /// Analyze bus factor risks
    BusFactor {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Ownership percentage threshold (default: 80)
        #[arg(short, long, default_value = "80.0")]
        threshold: f64,
    },
}

#[derive(Debug)]
pub struct Config {
    pub repo_path: String,
    pub author: String,
    pub since: String,
    pub show_diff: bool,
    pub play: bool,
    pub save_music_path: Option<String>,
    pub is_hotspots_command: bool,
    pub hotspots_path: Option<String>,
    pub who_knows_path: Option<String>,
    pub bus_factor_path: Option<String>,
    pub bus_factor_threshold: Option<f64>,
}

pub fn parse_cli_args() -> Config {
    let cli = Cli::parse();
    let author = cli.author.unwrap_or_else(get_git_user_name);

    let (is_hotspots_command, hotspots_path, who_knows_path, bus_factor_path, bus_factor_threshold, since) = match cli.command {
        Some(Commands::Hotspots { path, since }) => (true, path, None, None, None, since),
        Some(Commands::WhoKnows { path }) => (false, None, Some(path), None, None, "24 hours ago".to_string()),
        Some(Commands::BusFactor { path, threshold }) => (false, None, None, Some(path), Some(threshold), "24 hours ago".to_string()),
        None => (false, None, None, None, None, "24 hours ago".to_string()),
    };

    Config {
        repo_path: cli.repo_path,
        author,
        since,
        show_diff: cli.show_diff,
        play: cli.play,
        save_music_path: cli.save_music_path,
        is_hotspots_command,
        hotspots_path,
        who_knows_path,
        bus_factor_path,
        bus_factor_threshold,
    }
}

fn get_git_user_name() -> String {
    use std::process::Command;
    
    let output = Command::new("git")
        .arg("config")
        .arg("user.name")
        .output()
        .expect("Failed to execute git command");

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        panic!("Failed to get git user.name");
    }
} 