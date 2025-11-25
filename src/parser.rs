use crate::models::{DatabaseData, GossipEntry, NeighborhoodEdge, NodeData, TableData};
use anyhow::Result;
use flate2::read::GzDecoder;
use regex::Regex;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

pub fn scan_directory(input_dir: &Path) -> Result<HashMap<String, NodeData>> {
    let mut nodes = HashMap::new();
    let entries = fs::read_dir(input_dir)?;

    // First, check if we have subdirectories (nested structure)
    let mut file_entries = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
            
            // Heuristic: check if it looks like a node dir
            let current_log = path.join("MASQNode_rCURRENT.log");
            let has_logs = fs::read_dir(&path)?.any(|f| {
                f.map(|e| e.path().extension().is_some_and(|ext| ext == "zip"))
                 .unwrap_or(false)
            });

            if current_log.exists() || has_logs {
                match parse_node(&path) {
                    Ok(mut node_data) => {
                        node_data.name = dir_name.clone();
                        nodes.insert(dir_name, node_data);
                    }
                    Err(e) => eprintln!("Failed to parse node {}: {}", dir_name, e),
                }
            }
        } else {
            file_entries.push(path);
        }
    }

    // If no node subdirectories were found, try parsing as flat structure
    if nodes.is_empty() && !file_entries.is_empty() {
        // Group files by prefix (e.g., "1-MASQNode..." -> "1")
        let mut node_files: HashMap<String, Vec<PathBuf>> = HashMap::new();
        
        for path in file_entries {
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                // Look for prefix separated by hyphen
                if let Some(idx) = filename.find('-') {
                    let prefix = &filename[0..idx];
                    // Simple validation: prefix should be alphanumeric
                    if !prefix.is_empty() && prefix.chars().all(|c| c.is_alphanumeric()) {
                        node_files.entry(prefix.to_string()).or_default().push(path);
                    }
                }
            }
        }

        // Parse each group as a node
        for (node_name, files) in node_files {
            let mut node_data = NodeData {
                name: node_name.clone(),
                neighborhood: Vec::new(),
                gossip: Vec::new(),
                log_files: Vec::new(),
                current_log: String::new(),
                database: DatabaseData { tables: HashMap::new() },
            };

            // Process files for this node
            for path in files {
                let filename = path.file_name().unwrap().to_string_lossy();
                
                if filename.contains("MASQNode_rCURRENT.log") {
                    node_data.current_log = filename.to_string();
                    node_data.log_files.push(filename.to_string());
                    
                    // Parse log content
                    if let Ok(content) = read_last_lines(&path, 1000) { // Read initial chunk for parsing
                        parse_content(&content, &mut node_data);
                    }
                } else if filename.ends_with(".log") || filename.ends_with(".zip") {
                    node_data.log_files.push(filename.to_string());
                } else if filename.ends_with(".db") {
                    // Extract DB structure
                    if let Ok(db_data) = extract_database_structure(&path) {
                        node_data.database = db_data;
                    }
                }
            }
            
            // Only add if we found relevant data
            if !node_data.log_files.is_empty() || !node_data.database.tables.is_empty() {
                nodes.insert(node_name, node_data);
            }
        }
    }

    Ok(nodes)
}

fn parse_node(node_dir: &Path) -> Result<NodeData> {
    let node_name = node_dir.file_name().unwrap().to_string_lossy().to_string();
    let mut data = NodeData {
        name: node_name,
        neighborhood: Vec::new(),
        gossip: Vec::new(),
        log_files: Vec::new(),
        current_log: String::new(),
        database: DatabaseData::default(),
    };

    // Collect log files
    for entry in fs::read_dir(node_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.ends_with(".log") || file_name.ends_with(".log.zip") {
            data.log_files.push(file_name.clone());
        }
    }
    data.log_files.sort();

    // Process current log
    let current_log_path = node_dir.join("MASQNode_rCURRENT.log");
    if current_log_path.exists() {
        process_log_file(&current_log_path, &mut data)?;
        data.current_log = read_last_lines(&current_log_path, 1000)?;
    }

    // Process archived logs
    let log_files_clone = data.log_files.clone();
    for file_name in &log_files_clone {
        if file_name.ends_with(".log.zip") {
            let file_path = node_dir.join(file_name);
            if let Err(e) = process_zip_log(&file_path, &mut data) {
                eprintln!("Failed to process zip log {}: {}", file_name, e);
            }
        }
    }

    // Extract database data (structure only for now, or full if needed)
    // The original tool extracts EVERYTHING. For performance, let's just extract table names
    // and maybe small tables? Or just stick to the plan of on-demand.
    // However, the dashboard might need some data.
    // Let's implement the full extraction for compatibility first, but maybe optimize later.
    // Actually, the plan said "optimize this by only scanning the structure".
    // So let's do that.
    let db_path = node_dir.join("node-data.db");
    if db_path.exists() {
        if let Ok(db_data) = extract_database_structure(&db_path) {
            data.database = db_data;
        }
    }

    Ok(data)
}

fn process_log_file(path: &Path, data: &mut NodeData) -> Result<()> {
    let content = fs::read_to_string(path)?;
    parse_content(&content, data);
    Ok(())
}

fn process_zip_log(path: &Path, data: &mut NodeData) -> Result<()> {
    let file = File::open(path)?;
    let mut decoder = GzDecoder::new(file);
    let mut content = String::new();

    // Try to decode as gzip
    if decoder.read_to_string(&mut content).is_ok() {
        parse_content(&content, data);
    } else {
        // Fallback: try plain text
        let plain_content = fs::read_to_string(path)?;
        parse_content(&plain_content, data);
    }
    Ok(())
}

fn parse_content(content: &str, data: &mut NodeData) {
    let route_regex = Regex::new(r"Route back: (.*?) :").unwrap();
    let gossip_regex = Regex::new(r"^(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}) .*? (Neighborhood|GossipAcceptor): (Sent Gossip|Received Gossip|Current database): (digraph db \{ .* \})$").unwrap();

    for line in content.lines() {
        // Parse Neighborhood Routes
        if line.contains("DEBUG: Neighborhood: Route back:") {
            if let Some(caps) = route_regex.captures(line) {
                if let Some(route_str) = caps.get(1) {
                    let parts: Vec<&str> =
                        route_str.as_str().split(" -> ").map(|s| s.trim()).collect();
                    for i in 0..parts.len().saturating_sub(1) {
                        let from = parts[i].to_string();
                        let to = parts[i + 1].to_string();
                        if !data
                            .neighborhood
                            .iter()
                            .any(|e| e.from == from && e.to == to)
                        {
                            data.neighborhood.push(NeighborhoodEdge { from, to });
                        }
                    }
                }
            }
        }

        // Parse Gossip Graphs
        if line.contains("digraph db {") {
            if let Some(caps) = gossip_regex.captures(line) {
                data.gossip.push(GossipEntry {
                    timestamp: caps[1].to_string(),
                    actor: caps[2].to_string(),
                    tag: caps[3].to_string(),
                    dot: caps[4].to_string(),
                });
            }
        }
    }
}

fn read_last_lines(path: &Path, num_lines: usize) -> Result<String> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    let start = lines.len().saturating_sub(num_lines);
    Ok(lines[start..].join("\n"))
}

fn extract_database_structure(db_path: &Path) -> Result<DatabaseData> {
    let mut db_data = DatabaseData::default();

    // Copy to temp file to avoid locks
    let tmp_path = db_path.with_extension("db.tmp");
    fs::copy(db_path, &tmp_path)?;

    let conn = Connection::open(&tmp_path)?;

    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
    )?;
    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    drop(stmt); // Drop stmt before iterating

    for table_name in table_names {
        // For structure only, we leave rows empty
        // But we should get columns
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<_, _>>()?;
        drop(stmt); // Drop stmt before closing conn

        db_data.tables.insert(
            table_name,
            TableData {
                columns,
                rows: Vec::new(), // Empty rows for now
            },
        );
    }

    // Clean up
    let _ = conn.close();
    let _ = fs::remove_file(&tmp_path);
    let _ = fs::remove_file(format!("{}-wal", tmp_path.display()));
    let _ = fs::remove_file(format!("{}-shm", tmp_path.display()));

    Ok(db_data)
}

// Helper to get full table data on demand
pub fn get_table_data(db_path: &Path, table_name: &str) -> Result<TableData> {
    let conn = Connection::open(db_path)?; // Open readonly?

    // Get columns
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<_, _>>()?;

    // Get rows
    let mut stmt = conn.prepare(&format!("SELECT * FROM {}", table_name))?;
    let column_count = stmt.column_count();

    let rows = stmt
        .query_map([], |row| {
            let mut row_data = Vec::new();
            for i in 0..column_count {
                let val = row.get_ref(i)?;
                let json_val = match val {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(i) => serde_json::Value::Number(i.into()),
                    rusqlite::types::ValueRef::Real(f) => serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    rusqlite::types::ValueRef::Text(t) => {
                        serde_json::Value::String(String::from_utf8_lossy(t).to_string())
                    }
                    rusqlite::types::ValueRef::Blob(b) => {
                        serde_json::Value::String(format!("<BLOB {} bytes>", b.len()))
                    }
                };
                row_data.push(json_val);
            }
            Ok(row_data)
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(TableData { columns, rows })
}
