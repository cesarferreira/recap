use colored::*;
use std::fs::File;
use std::path::Path;

mod commands;
mod git;
mod ui;
mod music;

use commands::parse_cli_args;
use music::{MusicConfig, commit_to_note, generate_midi, play_midi};

fn main() {
    // Parse command line arguments
    let config = parse_cli_args();

    // Validate repository
    if let Err(e) = git::validate_repo(&config.repo_path) {
        eprintln!("{}", e.red());
        std::process::exit(1);
    }

    // Print initial summary
    println!(
        "{}",
        format!(
            "Recap of commits since '{}' by '{}' in '{}':\n",
            config.since.yellow(),
            config.author.green(),
            config.repo_path.blue()
        )
    );

    // Get and display commits
    let commits = git::get_commits(&config.repo_path, &config.author, &config.since, config.show_diff);
    let mut commit_notes = Vec::new();

    for commit in &commits {
        ui::print_commit(commit);

        if config.show_diff {
            if let Some(diff) = git::get_commit_diff(&config.repo_path, &commit.hash) {
                ui::print_diff(&diff);
            }
        }

        // Generate music notes if needed
        if config.generate_music || config.save_music_path.is_some() || config.play_music {
            let output = std::process::Command::new("git")
                .arg("-C")
                .arg(&config.repo_path)
                .arg("--no-pager")
                .arg("show")
                .arg("--numstat")
                .arg(&commit.hash)
                .output()
                .unwrap();

            if output.status.success() {
                let stats_output = String::from_utf8_lossy(&output.stdout);
                for stat_line in stats_output.lines() {
                    let parts: Vec<&str> = stat_line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let (Ok(add), Ok(del)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                            let file_ext = Path::new(parts[2])
                                .extension()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown");
                            
                            let mut note = commit_to_note(add, del, file_ext, &MusicConfig::default());
                            note.commit_hash = commit.hash.clone();
                            note.commit_msg = commit.message.clone();
                            note.file_path = parts[2].to_string();
                            commit_notes.push(note);
                        }
                    }
                }
            }
        }
    }

    // Get and display stats
    let stats = git::get_stats(&config.repo_path, &config.author, &config.since);
    ui::print_stats(&stats);

    // Handle music generation if requested
    if !commit_notes.is_empty() {
        let music_config = MusicConfig::default();
        let midi_with_notes = generate_midi(commit_notes, &music_config);

        // Handle playback first if requested
        if config.play_music {
            println!("\n{}", "🎵 Playing commit music...".green());
            if let Err(e) = play_midi(&midi_with_notes) {
                eprintln!("{}", format!("Error playing MIDI: {}", e).red());
            }
        }

        // Then save to specified file if requested
        if let Some(path) = &config.save_music_path {
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

            midi_with_notes.midi_data.write_std(&mut file).unwrap();
            println!("\n{}", format!("🎵 MIDI file saved to: {}", path).green());
        }
    }
}