# Contributing to Masq Log Visualizer

Thank you for your interest in contributing to Masq Log Visualizer! We welcome contributions from the community.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/yourusername/masq-log-visualizer.git
   cd masq-log-visualizer
   ```
3. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites
- Rust 1.70 or higher
- A modern web browser for testing

### Building and Running

```bash
# Run in development mode
cargo run -- -i /path/to/logs

# Run tests
cargo test

# Build release version
cargo build --release
```

## Making Changes

1. Make your changes in your feature branch
2. Test your changes thoroughly
3. Follow the existing code style
4. Add or update tests as needed
5. Update documentation if you're changing functionality

## Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and address any warnings

## Submitting Changes

1. **Commit your changes** with clear, descriptive commit messages
2. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```
3. **Open a Pull Request** on GitHub
4. Describe your changes clearly in the PR description
5. Wait for review and address any feedback

## Reporting Issues

- Use GitHub Issues to report bugs or request features
- Provide as much detail as possible
- Include steps to reproduce for bugs
- Include your environment details (OS, Rust version, etc.)

## Questions?

Feel free to open an issue for any questions about contributing!

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
