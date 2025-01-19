# Recap

> A beautiful Git commit history viewer with stats, colorful output, and musical visualization

Recap is a command-line tool that shows your Git commits across all branches with a beautiful, colorful interface. It includes commit statistics, optional diff viewing capabilities, and can generate musical representations of your commit history.

## ✨ Features

### 📊 Core Features
- 🎨 Colorful and easy-to-read commit history
- 📈 Commit statistics table
- 🌳 Shows commits from all branches
- 👥 Filter by author
- 🔍 Optional diff viewing
- ⏰ Flexible time range filtering

### 🔥 Code Analysis
- 📍 Identify code hotspots and high-churn files
- 👨‍💻 Find file experts with "who knows" analysis
- 🚌 Detect bus factor risks and knowledge silos
- 📊 Contributor statistics and suggestions
- ⚠️ Technical debt indicators

### 🎵 Musical Features
- 🎼 Musical visualization of commit history
- 🎹 MIDI generation from commits
- 🔊 Live playback support
- 💾 Save musical output to files

## 🚀 Installation

```bash
cargo install recap
```

## 💡 Usage

Basic usage (shows your commits from the last 24 hours):
```bash
recap
```

### 🎯 Core Commands

Show commits from the last week:
```bash
recap --since "1 week ago"
```

Show commits with diffs from a specific author:
```bash
recap --author "John Doe" --show-diff
```

View commits in a different repository:
```bash
recap --repo-path /path/to/repo --since "yesterday"
```

### 🔥 Code Analysis Commands

Analyze code hotspots in the entire repository:
```bash
$ recap hotspots
🔥 Hot Files (last month):
  1. src/api/users.rs (25 changes)
  2. src/db/schema.rs (18 changes)
```

Find who knows a specific file or directory best:
```bash
$ recap who-knows src/main.rs
📚 File Expertise:
  - Alice (65% - primary maintainer)
  - Bob (25%)
  - Charlie (10%)
```

Identify bus factor risks in the codebase:
```bash
$ recap bus-factor src/
High Risk (Bus Factor 1):
  - src/core/auth.rs (95% owned by Alice, 203 lines)
  - src/utils/crypto.rs (90% owned by Bob, 156 lines)
```

Options for bus factor analysis:
```bash
recap bus-factor              # analyze entire repo
recap bus-factor src/         # analyze specific directory
recap bus-factor --threshold 75   # custom ownership threshold (default: 80%)
```

This helps identify potential knowledge silos where:
- Files are predominantly owned by a single person
- There's risk if that person becomes unavailable
- Code might benefit from more shared ownership

### 🎵 Musical Commands

Generate and play commit history as music:
```bash
recap --generate-music --play
```

Save the musical representation to a file:
```bash
recap --generate-music --save-music output.midi
```

### 📝 Available Options

Core Options:
- `-a, --author <AUTHOR>` - Filter by author name/email (defaults to git config user.name)
- `-p, --repo-path <PATH>` - Path to Git repository (defaults to current directory)
- `-s, --since <TIME>` - How far back to look (defaults to "24 hours ago")
- `-d, --show-diff` - Show the diff for each commit

Hotspots Options:
- `--since <TIME>` - How far back to analyze (e.g. '2 weeks ago', 'all' for entire history)

Bus Factor Options:
- `--threshold <NUMBER>` - Ownership percentage threshold (default: 80)

Music Options:
- `-m, --generate-music` - Generate MIDI music from commit history
- `--save-music <FILE>` - Save generated music to a MIDI file
- `--play` - Play the generated music immediately

## 🎵 Musical Visualization Details

Recap can generate MIDI output that represents your commit history as musical notes:

- Additions are represented as ascending notes
- Deletions are represented as descending notes
- Different file changes are played with different instruments
- Commit size affects the volume of the notes

## 🛠️ Building from Source

1. Clone the repository
2. Run:
```bash
cargo build --release
```
3. The binary will be available in `target/release/recap`

## 📄 License

MIT License - feel free to use this in your own projects!
