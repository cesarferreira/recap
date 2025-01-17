#!/usr/bin/env bash

# Usage: ./show_commits.sh <github_username> <repo_path> [<time_range>]
# Examples:
#   ./show_commits.sh alice /path/to/your/repo
#   ./show_commits.sh alice /path/to/your/repo "2 days ago"

# Check for at least 2 arguments: username and repo path
if [ $# -lt 2 ]; then
  echo "Usage: $0 <github_username> <repo_path> [<time_range>]"
  exit 1
fi

GITHUB_USER=$1
REPO_PATH=$2
# If a third argument is provided, use that; otherwise default to "24 hours ago"
TIME_RANGE=${3:-"24 hours ago"}

# Ensure the provided path is a valid directory
if [ ! -d "$REPO_PATH" ]; then
  echo "Error: '$REPO_PATH' is not a valid directory."
  exit 1
fi

# Check if the provided path is inside a Git repository
inside_repo=$(git -C "$REPO_PATH" rev-parse --is-inside-work-tree 2>/dev/null)
if [ "$inside_repo" != "true" ]; then
  echo "Error: '$REPO_PATH' does not appear to be a Git repository (or any of its parent directories)."
  exit 1
fi

echo "Showing commits since '$TIME_RANGE' by user: $GITHUB_USER"
echo "Analyzing repository (including subfolder): $REPO_PATH"
echo

# Use --no-pager to prevent opening a pager like less
git -C "$REPO_PATH" --no-pager log \
    --author="$GITHUB_USER" \
    --since="$TIME_RANGE" \
    --pretty=format:"%h - %s [%cr by %an]"

exit 0