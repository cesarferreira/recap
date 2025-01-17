use std::path::Path;
use std::process::Command;
use clap::{Arg, Command as ClapCommand};

fn main() {
    // 1. Define the CLI structure using Clap
    let matches = ClapCommand::new("show_commits")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Shows commits from a specified author in a Git repo, within a given time range.")
        .arg(
            Arg::new("author")
                .long("author")
                .short('a')
                .value_name("AUTHOR")
                .help("Specifies the author name or email to filter by.")
                .required(true),
        )
        .arg(
            Arg::new("repo_path")
                .long("repo-path")
                .short('p')
                .value_name("REPO_PATH")
                .help("Specifies the path to the Git repository (can be a subfolder).")
                .required(true),
        )
        .arg(
            Arg::new("since")
                .long("since")
                .short('s')
                .value_name("SINCE")
                .help("Specifies how far back to go for commits. Defaults to 24 hours ago.")
                .default_value("24 hours ago")
                .required(false),
        )
        .get_matches();

    // 2. Extract the arguments
    let author = matches.get_one::<String>("author").unwrap();
    let repo_path = matches.get_one::<String>("repo_path").unwrap();
    let since = matches.get_one::<String>("since").unwrap();

    // 3. Validate the repo_path is a directory
    if !Path::new(repo_path).is_dir() {
        eprintln!("Error: '{}' is not a valid directory.", repo_path);
        std::process::exit(1);
    }

    // 4. Check if the path is inside a Git repository
    //    Using `git -C <repo_path> rev-parse --is-inside-work-tree`
    let inside_repo_check = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output();

    match inside_repo_check {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Error: '{}' is not a Git repository.", repo_path);
                std::process::exit(1);
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim() != "true" {
                    eprintln!("Error: '{}' is not a valid Git repository.", repo_path);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error running `git rev-parse`: {e}");
            std::process::exit(1);
        }
    }

    // 5. Run the `git log` command with the specified arguments
    println!(
        "Showing commits since '{}' by author '{}' in repo: {}\n",
        since, author, repo_path
    );

    let git_log_output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("--no-pager")  // Prevent opening a pager like 'less'
        .arg("log")
        .arg(format!("--author={}", author))
        .arg(format!("--since={}", since))
        .arg("--pretty=format:%h - %s [%cr by %an]")
        .output();

    match git_log_output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // If there's no output, you either have no matching commits or are outside the date range
            if stdout.trim().is_empty() {
                println!(
                    "No commits found matching author '{}' since '{}'.",
                    author, since
                );
            } else {
                println!("{}", stdout);
            }
        }
        Err(e) => {
            eprintln!("Error running `git log`: {e}");
            std::process::exit(1);
        }
    }
}