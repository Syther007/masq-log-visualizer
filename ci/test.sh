#!/usr/bin/env bash
# Test script - runs the test suite

set -e

echo "Running tests..."
cargo test --verbose

echo "✅ All tests passed!"
