# Masq Log Visualizer

Masq Node Log Visualizer. This tool parses Masq node logs and SQLite databases, serving a web interface to visualize logs, database tables, and gossip graphs.

## Features

- **Dashboard**: Overview of all processed nodes
- **Log Viewer**: Filterable, paginated log viewing with download links
- **Database Explorer**: Browse SQLite database tables with search and filtering
- **Gossip Graph Visualization**: Interactive visualization of "Gossip" DOT graphs with time-travel slider

## Prerequisites

- Rust 1.70+ (tested with 1.79.0)
- Any system with  a modern web browser

## Quick Start

### Run in Development Mode

```bash
cargo run -- -i ../Example-Logs-3
```

Then open http://127.0.0.1:3000 in your browser.

### Build Release Binary

```bash
cargo build --release
```

The binary will be at `target/release/masq-log-visualizer`.

## Usage

```bash
masq-log-visualizer -i <input_directory> [OPTIONS]
```

### Options

- `-i, --input <PATH>` - Input directory containing node folders (required)
- `-p, --port <PORT>` - Server port (default: 3000)
- `--host <HOST>` - Server host (default: 127.0.0.1)
- `-h, --help` - Print help information
- `-V, --version` - Print version information

### Example

```bash
# Run on default port 3000
cargo run -- -i ../Example-Logs-3

# Run on custom port
cargo run -- -i ../Example-Logs-3 --port 8080

# Bind to all interfaces
cargo run -- -i ../Example-Logs-3 --host 0.0.0.0 --port 8080
```

## Project Structure

```
├── src/
│   ├── main.rs       # Entry point, CLI, server setup
│   ├── models.rs     # Data structures
│   ├── parser.rs     # Log parsing and database extraction
│   └── routes.rs     # Web server route handlers
├── templates/
│   ├── dashboard.html    # Main dashboard (Tera template)
│   └── node_view.html    # Node detail view (Tera template)
├── assets/
│   └── vis-network.min.js  # Vis.js for graph visualization
└── Cargo.toml        # Dependencies
```

## API Endpoints

- `GET /` - Dashboard view
- `GET /node/:node_name` - Node detail view
- `GET /api/logs/:node_name/:file_name/range?fromEnd=true&lines=1000` - Paginated log content
- `GET /api/logs/:node_name/:file_name` - Download log file
- `GET /api/db/:node_name` - Database table list
- `GET /api/db/:node_name/:table_name` - Fetch table data on-demand
- `GET /api/gossip/:node_name` - Gossip graph data
- `GET /assets/*` - Static assets

## Key Improvements Over Original

### Performance
- **On-Demand Database Loading**: Only loads table structure on startup; actual row data is fetched via API when needed
- **Native Compiled Code**: Significantly faster than interpreted JavaScript
- **Efficient Memory Management**: No GC pauses

### Technology Stack
- **Axum 0.7**: Modern, type-safe web framework
- **Tera 1.19**: Powerful template engine (similar to Jinja2)
- **Rusqlite 0.31**: SQLite with bundled library
- **Tokio**: High-performance async runtime

## Building and Releases

### Automated Builds

This project uses GitHub Actions to automatically build binaries for multiple platforms.

**Supported Platforms:**
- Linux (x64, ARM64)
- macOS (x64, ARM64)  
- Windows (x64, ARM64)

### Creating a Release

To create a new release with binaries:

1. Tag your commit:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

2. GitHub Actions will automatically:
   - Build binaries for all platforms
   - Package them with templates and assets
   - Create a GitHub release with downloadable archives

### Manual Trigger

You can manually trigger builds from the GitHub Actions tab.

### CI/CD

- **CI Workflow**: Runs tests, formatting checks, and clippy on every push/PR
- **Release Workflow**: Builds cross-platform binaries when tags are pushed

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.