# CI Scripts

This directory contains scripts to run CI checks locally before pushing to GitHub.

## Available Scripts

### `test.sh`
Runs the test suite:
```bash
./ci/test.sh
```

### `fmt.sh`
Checks code formatting:
```bash
./ci/fmt.sh
```

### `clippy.sh`
Runs clippy lints (with warnings as errors):
```bash
./ci/clippy.sh
```

### `check-all.sh`
Runs all CI checks in sequence:
```bash
./ci/check-all.sh
```

## Making Scripts Executable

If needed, make the scripts executable:
```bash
chmod +x ci/*.sh
```

## Pre-commit Hook

To automatically run checks before committing, you can create a git hook:

```bash
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
./ci/check-all.sh
EOF

chmod +x .git/hooks/pre-commit
```

## Fix Formatting Issues

If formatting checks fail, auto-fix with:
```bash
cargo fmt
```
