#!/usr/bin/env bash
# Clippy script - runs linting checks

set -e

echo "Running clippy lints..."
cargo clippy -- -D warnings

echo "✅ No clippy warnings!"
