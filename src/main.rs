use clap::{Arg, Command as ClapCommand};
use colored::*;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::{Command as ProcessCommand, Stdio};
mod midi;
use midi::{CommitNote, MusicConfig, commit_to_note, generate_midi, play_midi};

// Tabled imports
use tabled::{
    Table, Tabled,
    Style, Disable,
    Modify,
    object::Segment,
    Alignment
};

fn main() {
    // 1. Define the CLI with Clap
    let matches = ClapCommand::new("recap")
        .version("1.2.0")
        .author("Your Name <your.email@example.com>")
        .about("Shows your commits (all branches) in color, plus a stats table.")
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

    // 2. Extract arguments (or use defaults)
    let since = matches
        .get_one::<String>("since")
        .unwrap_or(&"24 hours ago".to_string())
        .clone();

    // If no repo_path is specified, use current dir
    let repo_path = match matches.get_one::<String>("repo_path") {
        Some(path) => path.clone(),
        None => match env::current_dir() {
            Ok(dir) => dir.display().to_string(),
            Err(e) => {
                eprintln!("{}", format!("Error getting current directory: {e}").red());
                std::process::exit(1);
            }
        },
    };

    // If no author is specified, read from local git config user.name
    let author = match matches.get_one::<String>("author") {
        Some(a) => a.clone(),
        None => {
            let output = ProcessCommand::new("git")
                .arg("-C")
                .arg(&repo_path)
                .arg("config")
                .arg("user.name")
                .output();

            match output {
                Ok(o) => {
                    if o.status.success() {
                        let name = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        if name.is_empty() {
                            eprintln!(
                                "{}",
                                "No author specified and user.name is empty in git config.".red()
                            );
                            std::process::exit(1);
                        }
                        name
                    } else {
                        eprintln!(
                            "{}",
                            "No author specified and failed to get user.name from git config."
                                .red()
                        );
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("{}", format!("Failed to run `git config user.name`: {e}").red());
                    std::process::exit(1);
                }
            }
        }
    };

    // 3. Validate that repo_path is a directory
    if !Path::new(&repo_path).is_dir() {
        eprintln!(
            "{}",
            format!("Error: '{repo_path}' is not a valid directory.").red()
        );
        std::process::exit(1);
    }

    // 4. Check if the path is inside a Git repository
    let inside_repo_check = ProcessCommand::new("git")
        .arg("-C")
        .arg(&repo_path)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output();

    match inside_repo_check {
        Ok(output) => {
            if !output.status.success() {
                eprintln!(
                    "{}",
                    format!("Error: '{repo_path}' is not a Git repository.").red()
                );
                std::process::exit(1);
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim() != "true" {
                    eprintln!(
                        "{}",
                        format!("Error: '{repo_path}' is not a valid Git repository.").red()
                    );
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Error running `git rev-parse`: {e}").red());
            std::process::exit(1);
        }
    }

    // 5. Print an initial summary
    println!(
        "{}",
        format!(
            "Recap of commits since '{}' by '{}' in '{}':\n",
            since.yellow(),
            author.green(),
            repo_path.blue()
        )
    );

    // ---------------------------------------------------------------------
    // PART A: Print each commit line in color
    // ---------------------------------------------------------------------
    let show_diff = matches.get_flag("show_diff");
    let format = if show_diff {
        "%h - %s [%cr by %an]%n"  // Add newline after each commit when showing diff
    } else {
        "%h - %s [%cr by %an]"
    };

    let mut child = match ProcessCommand::new("git")
        .arg("-C")
        .arg(&repo_path)
        .arg("--no-pager")
        .arg("log")
        .arg("--all") // includes all branches
        .arg(format!("--author={}", author))
        .arg(format!("--since={}", since))
        .arg(format!("--pretty=format:{}", format))
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", format!("Error running `git log`: {e}").red());
            std::process::exit(1);
        }
    };

    // Regex to parse lines like: <hash> - <message> [<relative time> by <author>]
    let re = Regex::new(r"^([0-9a-f]+) - (.*?) \[(.*?) by (.*?)\]$").unwrap();

    let generate_music = matches.get_flag("generate_music");
    let save_music_path = matches.get_one::<String>("save_music");
    let mut commit_notes = Vec::new();

    // Process commits
    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line_str) = line {
                if let Some(caps) = re.captures(&line_str) {
                    let short_hash = caps.get(1).unwrap().as_str();
                    let commit_msg = caps.get(2).unwrap().as_str();
                    let relative_time = caps.get(3).unwrap().as_str();
                    let commit_author = caps.get(4).unwrap().as_str();

                    println!(
                        "{} - {} [{} by {}]",
                        short_hash.yellow().bold(),
                        commit_msg.cyan(),
                        relative_time.green(),
                        commit_author.magenta()
                    );

                    // Generate music if enabled
                    if generate_music || save_music_path.is_some() || matches.get_flag("play") {
                        // Get the diff stats for the commit
                        let diff_stats = ProcessCommand::new("git")
                            .arg("-C")
                            .arg(&repo_path)
                            .arg("--no-pager")
                            .arg("show")
                            .arg("--numstat")
                            .arg(short_hash)
                            .output()
                            .unwrap();

                        if diff_stats.status.success() {
                            let stats_output = String::from_utf8_lossy(&diff_stats.stdout);
                            for stat_line in stats_output.lines() {
                                let parts: Vec<&str> = stat_line.split_whitespace().collect();
                                if parts.len() >= 3 {
                                    if let (Ok(add), Ok(del)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                                        let file_ext = Path::new(parts[2])
                                            .extension()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("unknown");
                                        
                                        let note = commit_to_note(add, del, file_ext, &MusicConfig::default());
                                        commit_notes.push(note);
                                    }
                                }
                            }
                        }
                    }

                    // Show diff if enabled
                    if show_diff {
                        let diff_child = ProcessCommand::new("git")
                            .arg("-C")
                            .arg(&repo_path)
                            .arg("--no-pager")
                            .arg("show")
                            .arg("--color=always")
                            .arg("--stat")
                            .arg("--patch")
                            .arg(short_hash)
                            .stdout(Stdio::piped())
                            .spawn();

                        if let Ok(mut diff_child) = diff_child {
                            if let Some(diff_stdout) = diff_child.stdout.take() {
                                let diff_reader = std::io::BufReader::new(diff_stdout);
                                for diff_line in diff_reader.lines() {
                                    if let Ok(diff_line) = diff_line {
                                        println!("    {}", diff_line);
                                    }
                                }
                            }
                            println!(); // Add a blank line after each diff
                        }
                    }
                } else {
                    println!("{}", line_str);
                }
            }
        }
    }

    if let Err(e) = child.wait() {
        eprintln!("{}", format!("Error waiting for `git log` to finish: {e}").red());
        std::process::exit(1);
    }

    // ---------------------------------------------------------------------
    // PART B: Gather stats (#commits, lines added, lines deleted)
    // ---------------------------------------------------------------------
    let mut stats_child = match ProcessCommand::new("git")
        .arg("-C")
        .arg(&repo_path)
        .arg("log")
        .arg("--all")
        .arg(format!("--author={}", author))
        .arg(format!("--since={}", since))
        // We'll use a special format + numstat
        .arg("--pretty=tformat:COMMIT")
        .arg("--numstat")
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", format!("Error gathering stats with `git log`: {e}").red());
            std::process::exit(1);
        }
    };

    let mut commits_count = 0;
    let mut total_additions = 0;
    let mut total_deletions = 0;

    if let Some(stdout) = stats_child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let line_str = line.trim().to_string();
            // "COMMIT" indicates a new commit
            if line_str == "COMMIT" {
                commits_count += 1;
            } else if !line_str.is_empty() {
                // Lines of the form "<added> <deleted> <filename>"
                // Could be: "100 50 src/main.rs" or "-   -  some_binary"
                let parts: Vec<&str> = line_str.split_whitespace().collect();
                if parts.len() >= 3 {
                    // Attempt to parse additions/deletions
                    let added = parts[0];
                    let deleted = parts[1];

                    // If they are "-", it's likely a binary or rename; treat them as zero.
                    let added_num = added.parse::<i32>().unwrap_or(0);
                    let deleted_num = deleted.parse::<i32>().unwrap_or(0);

                    total_additions += added_num;
                    total_deletions += deleted_num;
                }
            }
        }
    }

    if let Err(e) = stats_child.wait() {
        eprintln!(
            "{}",
            format!("Error waiting for `git log --numstat` to finish: {e}").red()
        );
        std::process::exit(1);
    }

    // ---------------------------------------------------------------------
    // PART C: Print stats in a Tabled table with your color scheme
    // ---------------------------------------------------------------------

    // 1. Create a struct for tabled
    #[derive(Tabled)]
    struct StatsRow {
        label: String, // String so we can apply .bold()
        value: String,
    }

    // 2. Build a vector of rows, making left label bold
    let stats_data = vec![
        StatsRow {
            label: "Commits".bold().to_string(),
            value: commits_count
                .to_string()
                .yellow()
                .bold()
                .to_string(),
        },
        StatsRow {
            label: "Lines added (+)".bold().to_string(),
            value: total_additions
                .to_string()
                .green()
                .bold()
                .to_string(),
        },
        StatsRow {
            label: "Lines deleted (-)".bold().to_string(),
            value: total_deletions
                .to_string()
                .red()
                .bold()
                .to_string(),
        },
    ];

    // 3. Build the table
    let table = Table::new(stats_data)
        // Use straight lines:
        .with(Style::modern())
        // Disable header row
        .with(Disable::Row(..1))
        // Align cells to the left
        .with(Modify::new(Segment::all()).with(Alignment::left()));

    println!();
    println!("{}", "====================== STATS ======================".bold());
    println!();
    println!("{table}");

    // After processing all commits, generate and save/play music if requested
    if !commit_notes.is_empty() {
        let config = MusicConfig::default();
        let midi_data = generate_midi(commit_notes, &config);

        // Handle playback first if requested
        if matches.get_flag("play") {
            println!("\n{}", "🎵 Playing commit music...".green());
            if let Err(e) = play_midi(&midi_data) {
                eprintln!("{}", format!("Error playing MIDI: {}", e).red());
            }
        }

        // Then save to specified file if requested
        if let Some(path) = save_music_path {
            // Create parent directories if they don't exist
            if let Some(parent) = Path::new(path).parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("{}", format!("Error creating directories: {}", e).red());
                    std::process::exit(1);
                }
            }
            let mut file = match File::create(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}", format!("Error creating file: {}", e).red());
                    std::process::exit(1);
                }
            };
            midi_data.write_std(&mut file).unwrap();
            println!("\n{}", format!("🎵 MIDI file saved to: {}", path).green());
        }
    }
}