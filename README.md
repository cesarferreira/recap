# 📊 Recap

> A beautiful Git commit history viewer with stats and colorful output

Recap is a command-line tool that shows your Git commits across all branches with a beautiful, colorful interface. It includes commit statistics and optional diff viewing capabilities.

## ✨ Features

- 🎨 Colorful and easy-to-read commit history
- 📊 Commit statistics table
- 🌳 Shows commits from all branches
- 👥 Filter by author
- 🔍 Optional diff viewing
- ⏰ Flexible time range filtering

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

## 🛠️ Building from Source

1. Clone the repository
2. Run:
```bash
cargo build --release
```
3. The binary will be available in `target/release/recap`

## 📄 License

MIT License - feel free to use this in your own projects!
