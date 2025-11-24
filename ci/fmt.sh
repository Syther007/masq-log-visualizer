#!/usr/bin/env bash
# Format check script - verifies code formatting

set -e

echo "Checking code formatting..."
cargo fmt --check

echo "✅ Code formatting is correct!"
