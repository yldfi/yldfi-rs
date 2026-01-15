#!/bin/sh
# Install git hooks for this repository

set -e

HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HOOK_DIR/.." && pwd)"

echo "Installing git hooks..."
git -C "$REPO_ROOT" config core.hooksPath .githooks

echo "Git hooks installed!"
echo "Pre-commit will now run: fmt check, clippy, and tests"
