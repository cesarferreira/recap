# Recap

> A beautiful Git commit history viewer with stats, colorful output, and musical visualization

Recap is a command-line tool that shows your Git commits across all branches with a beautiful, colorful interface. It includes commit statistics, optional diff viewing capabilities, and can generate musical representations of your commit history.

## ✨ Features

- 🎨 Colorful and easy-to-read commit history
- 📊 Commit statistics table
- 🌳 Shows commits from all branches
- 👥 Filter by author
- 🔍 Optional diff viewing
- ⏰ Flexible time range filtering
- 🎵 Musical visualization of commit history

## 🚀 Installation

```bash
cargo install recap
```

## 💡 Usage

Basic usage (shows your commits from the last 24 hours):
```bash
recap
```

### 🎯 Options

- `-a, --author <AUTHOR>` - Filter by author name/email (defaults to git config user.name)
- `-p, --repo-path <PATH>` - Path to Git repository (defaults to current directory)
- `-s, --since <TIME>` - How far back to look (defaults to "24 hours ago")
- `-d, --show-diff` - Show the diff for each commit
- `-m, --generate-music` - Generate MIDI music from commit history
- `--save-music <FILE>` - Save generated music to a MIDI file
- `--play` - Play the generated music immediately

### 📝 Examples

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

## 🎵 Musical Visualization

Recap can generate MIDI output that represents your commit history as musical notes:

- Additions are represented as ascending notes
- Deletions are represented as descending notes
- Different file changes are played with different instruments
- Commit size affects the volume of the notes

To generate and play the music immediately:
```bash
recap --generate-music --play
```

To save the music to a MIDI file:
```bash
recap --generate-music --save-music output.midi
```

## 🛠️ Building from Source

1. Clone the repository
2. Run:
```bash
cargo build --release
```
3. The binary will be available in `target/release/recap`

## 📄 License

MIT License - feel free to use this in your own projects!
