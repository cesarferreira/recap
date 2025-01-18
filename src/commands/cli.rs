use clap::{Arg, Command as ClapCommand};

pub struct CliConfig {
    pub author: String,
    pub repo_path: String,
    pub since: String,
    pub show_diff: bool,
    pub generate_music: bool,
    pub save_music_path: Option<String>,
    pub play_music: bool,
    pub who_knows_path: Option<String>,
}

pub fn parse_cli_args() -> CliConfig {
    let matches = ClapCommand::new("recap")
        .version("1.2.0")
        .author("Your Name <your.email@example.com>")
        .about("Shows your commits (all branches) in color, plus a stats table.")
        .subcommand(
            ClapCommand::new("who-knows")
                .about("Shows who has the most expertise with a file or directory")
                .arg(
                    Arg::new("path")
                        .help("Path to the file or directory to analyze")
                        .required(true)
                        .index(1),
                ),
        )
        .arg(
            Arg::new("author")
                .long("author")
                .short('a')
                .value_name("AUTHOR")
                .help("Author name/email to filter by. Defaults to git config user.name if not provided.")
                .required(false),
        )
        .arg(
            Arg::new("repo_path")
                .long("repo-path")
                .short('p')
                .value_name("REPO_PATH")
                .help("Path to the Git repo (can be subfolder). Defaults to current directory if not provided.")
                .required(false),
        )
        .arg(
            Arg::new("since")
                .long("since")
                .short('s')
                .value_name("SINCE")
                .help("How far back to go for commits. Defaults to '24 hours ago'.")
                .default_value("24 hours ago")
                .required(false),
        )
        .arg(
            Arg::new("show_diff")
                .long("show-diff")
                .short('d')
                .help("Show the diff for each commit")
                .action(clap::ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("generate_music")
                .long("generate-music")
                .short('m')
                .help("Generate MIDI music from commit history")
                .action(clap::ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("save_music")
                .long("save-music")
                .value_name("FILE")
                .help("Save generated music to a MIDI file")
                .required(false),
        )
        .arg(
            Arg::new("play")
                .long("play")
                .help("Play the generated music immediately")
                .action(clap::ArgAction::SetTrue)
                .required(false),
        )
        .get_matches();

    let who_knows_path = if let Some(who_knows_matches) = matches.subcommand_matches("who-knows") {
        who_knows_matches.get_one::<String>("path").map(|s| s.to_string())
    } else {
        None
    };

    CliConfig {
        since: matches
            .get_one::<String>("since")
            .unwrap_or(&"24 hours ago".to_string())
            .clone(),
        repo_path: matches.get_one::<String>("repo_path")
            .map(|s| s.to_string())
            .unwrap_or_else(|| std::env::current_dir()
                .expect("Failed to get current directory")
                .display()
                .to_string()),
        author: matches.get_one::<String>("author")
            .map(|s| s.to_string())
            .unwrap_or_else(get_git_user_name),
        show_diff: matches.get_flag("show_diff"),
        generate_music: matches.get_flag("generate_music"),
        save_music_path: matches.get_one::<String>("save_music").map(|s| s.to_string()),
        play_music: matches.get_flag("play"),
        who_knows_path,
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