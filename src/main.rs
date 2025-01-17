use clap::{Arg, Command as ClapCommand};
use colored::*;
use regex::Regex;
use std::env;
use std::io::BufRead;
use std::path::Path;
use std::process::{Command as ProcessCommand, Stdio};

fn main() {
    // 1. Define the CLI with Clap
    let matches = ClapCommand::new("recap")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Shows your commits in color. Default is 'yesterday' or the last 24 hours.")
        .arg(
            Arg::new("author")
                .long("author")
                .short('a')
                .value_name("AUTHOR")
                .help("Author name or email to filter by. Defaults to git config user.name if not provided.")
                .required(false),
        )
        .arg(
            Arg::new("repo_path")
                .long("repo-path")
                .short('p')
                .value_name("REPO_PATH")
                .help("Path to the Git repository (or subfolder). Defaults to current directory if not provided.")
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

    // 5. Print an initial summary (in color!)
    println!(
        "{}",
        format!(
            "Recap of commits since '{}' by '{}' in '{}':\n",
            since.yellow(),
            author.green(),
            repo_path.blue()
        )
    );

    // 6. Prepare to run the git log command
    // We'll capture stdout so we can parse lines for colorization
    let mut child = match ProcessCommand::new("git")
        .arg("-C")
        .arg(&repo_path)
        .arg("--no-pager")
        .arg("log")
        .arg(format!("--author={}", author))
        .arg(format!("--since={}", since))
        .arg("--pretty=format:%h - %s [%cr by %an]")
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", format!("Error running `git log`: {e}").red());
            std::process::exit(1);
        }
    };

    // 7. Regex to parse: <hash> - <message> [<relative time> by <author>]
    let re = Regex::new(r"^([0-9a-f]+) - (.*?) \[(.*?) by (.*?)\]$").unwrap();

    // 8. Read stdout line by line
    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line_str) = line {
                // If line matches the pattern, color each group separately
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
                } else {
                    // If for some reason the line doesn't match, just print it
                    println!("{}", line_str);
                }
            }
        }
    }

    // 9. Wait for the child process to exit
    if let Err(e) = child.wait() {
        eprintln!("{}", format!("Error waiting for `git log` to finish: {e}").red());
        std::process::exit(1);
    }
}