#!/usr/bin/env bash
# Run all CI checks locally

set -e

echo "=========================================="
echo "Running all CI checks..."
echo "=========================================="
echo ""

# Run tests
echo "1️⃣  Running tests..."
./ci/test.sh
echo ""

# Check formatting
echo "2️⃣  Checking formatting..."
./ci/fmt.sh
echo ""

# Run clippy
echo "3️⃣  Running clippy..."
./ci/clippy.sh
echo ""

echo "=========================================="
echo "✅ All CI checks passed!"
echo "=========================================="
