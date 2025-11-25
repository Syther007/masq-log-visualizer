use crate::models::{AllNodesData, NodeData};
use crate::parser::get_table_data;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tera::{Context, Tera};

#[derive(Clone)]
pub struct AppState {
    pub nodes_data: Arc<RwLock<AllNodesData>>,
    pub input_dir: PathBuf,
    pub tera: Arc<Tera>,
}

#[derive(Deserialize)]
pub struct LogRangeParams {
    pub start: Option<usize>,
    pub lines: Option<usize>,
    #[serde(rename = "fromEnd")]
    pub from_end: Option<String>, // "true" or "false"
}

#[derive(Serialize)]
pub struct LogResponse {
    pub lines: Vec<String>,
    #[serde(rename = "totalLines")]
    pub total_lines: usize,
    pub start: usize,
    pub end: usize,
}

// --- API Handlers ---

pub async fn get_log_range(
    State(state): State<AppState>,
    Path((node_name, file_name)): Path<(String, String)>,
    Query(params): Query<LogRangeParams>,
) -> impl IntoResponse {
    // Look up the correct path from NodeData
    let nodes = state.nodes_data.read().unwrap();
    let log_path = if let Some(node) = nodes.get(&node_name) {
        // Find matching log file path
        let stored_path = node.log_files.iter()
            .find(|path| {
                PathBuf::from(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == file_name)
                    .unwrap_or(false)
            });
        
        if let Some(path_str) = stored_path {
            let path = PathBuf::from(path_str);
            // Check if this is a full/relative path or just a filename
            if path.parent().is_some() && path.parent() != Some(std::path::Path::new("")) {
                // It's a path with directory components (flat structure)
                path
            } else {
                // It's just a filename (nested structure), construct full path
                state.input_dir.join(&node_name).join(&file_name)
            }
        } else {
            // Not found in log_files, fallback to constructed path
            state.input_dir.join(&node_name).join(&file_name)
        }
    } else {
        // Node not found, fallback to constructed path
        state.input_dir.join(&node_name).join(&file_name)
    };
    drop(nodes);

    if !log_path.exists() {
        return (axum::http::StatusCode::NOT_FOUND, "Log file not found").into_response();
    }

    let file = match File::open(&log_path) {
        Ok(f) => f,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to open log file",
            )
                .into_response()
        }
    };

    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
    let total_lines = all_lines.len();

    let num_lines = params.lines.unwrap_or(1000);
    let from_end = params.from_end.as_deref() == Some("true");

    let (start, end, lines) = if from_end {
        let start = total_lines.saturating_sub(num_lines);
        let end = total_lines;
        let lines = all_lines[start..end].to_vec();
        (start, end, lines)
    } else {
        let start = params.start.unwrap_or(0);
        let end = (start + num_lines).min(total_lines);
        let lines = if start < total_lines {
            all_lines[start..end].to_vec()
        } else {
            Vec::new()
        };
        (start, end, lines)
    };

    Json(LogResponse {
        lines,
        total_lines,
        start,
        end,
    })
    .into_response()
}

pub async fn download_log(
    State(state): State<AppState>,
    Path((node_name, file_name)): Path<(String, String)>,
) -> impl IntoResponse {
    // Look up the correct path from NodeData
    let nodes = state.nodes_data.read().unwrap();
    let log_path = if let Some(node) = nodes.get(&node_name) {
        // Find matching log file path
        let stored_path = node.log_files.iter()
            .find(|path| {
                PathBuf::from(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == file_name)
                    .unwrap_or(false)
            });
        
        if let Some(path_str) = stored_path {
            let path = PathBuf::from(path_str);
            // Check if this is a full/relative path or just a filename
            if path.parent().is_some() && path.parent() != Some(std::path::Path::new("")) {
                // It's a path with directory components (flat structure)
                path
            } else {
                // It's just a filename (nested structure), construct full path
                state.input_dir.join(&node_name).join(&file_name)
            }
        } else {
            // Not found in log_files, fallback to constructed path
            state.input_dir.join(&node_name).join(&file_name)
        }
    } else {
        // Node not found, fallback to constructed path
        state.input_dir.join(&node_name).join(&file_name)
    };
    drop(nodes);

    if !log_path.exists() {
        return (axum::http::StatusCode::NOT_FOUND, "Log file not found").into_response();
    }

    // Extract just the filename for the attachment header
    let attachment_name = log_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&file_name);

    // Serve file as attachment
    match std::fs::read(&log_path) {
        Ok(bytes) => (
            [
                (axum::http::header::CONTENT_TYPE, "text/plain"),
                (
                    axum::http::header::CONTENT_DISPOSITION,
                    &format!("attachment; filename=\"{}\"", attachment_name),
                ),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read file",
        )
            .into_response(),
    }
}

pub async fn get_db_tables(
    State(state): State<AppState>,
    Path(node_name): Path<String>,
) -> impl IntoResponse {
    let nodes = state.nodes_data.read().unwrap();
    if let Some(node) = nodes.get(&node_name) {
        // We return the structure we have in memory (which is just empty tables with names)
        // OR we could re-scan. The memory one has table names.
        Json(&node.database.tables).into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}

pub async fn get_db_table_data(
    State(state): State<AppState>,
    Path((node_name, table_name)): Path<(String, String)>,
) -> impl IntoResponse {
    // Try nested structure first
    let mut db_path = state.input_dir.join(&node_name).join("node-data.db");
    
    // If nested structure database doesn't exist, try flat structure
    if !db_path.exists() {
        // Flat structure: {node_name}-node-data.db in input_dir
        db_path = state.input_dir.join(format!("{}-node-data.db", node_name));
    }

    if !db_path.exists() {
        return (axum::http::StatusCode::NOT_FOUND, "Database not found").into_response();
    }

    match get_table_data(&db_path, &table_name) {
        Ok(data) => Json(data).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query database: {}", e),
        )
            .into_response(),
    }
}

pub async fn get_gossip(
    State(state): State<AppState>,
    Path(node_name): Path<String>,
) -> impl IntoResponse {
    let nodes = state.nodes_data.read().unwrap();
    if let Some(node) = nodes.get(&node_name) {
        Json(&node.gossip).into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}

// --- View Handlers ---

pub async fn dashboard(State(state): State<AppState>) -> impl IntoResponse {
    let nodes = state.nodes_data.read().unwrap();
    let mut context = Context::new();

    // Convert HashMap to Vec for template
    let nodes_vec: Vec<&NodeData> = nodes.values().collect();
    let mut all_nodes: Vec<&String> = nodes.keys().collect();
    all_nodes.sort(); // Sort alphabetically

    context.insert("nodes", &nodes_vec);
    context.insert("allNodes", &all_nodes);
    context.insert("inputDir", &state.input_dir.to_string_lossy());

    // We also need fileTree for the dashboard...
    // Implementing a simple file tree structure
    let file_tree = get_directory_tree(&state.input_dir);
    context.insert("fileTree", &file_tree);

    match state.tera.render("dashboard.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {}", e),
        )
            .into_response(),
    }
}

pub async fn node_view(
    State(state): State<AppState>,
    Path(node_name): Path<String>,
) -> impl IntoResponse {
    let nodes = state.nodes_data.read().unwrap();

    if let Some(node) = nodes.get(&node_name) {
        let mut context = Context::new();
        let mut all_nodes: Vec<&String> = nodes.keys().collect();
        all_nodes.sort(); // Sort alphabetically

        // Extract the filename from the first log file path
        let current_log_file = node.log_files.first()
            .and_then(|path| {
                PathBuf::from(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "MASQNode_rCURRENT.log".to_string());

        context.insert("node", node);
        context.insert("allNodes", &all_nodes);
        context.insert("currentLogFile", &current_log_file);

        match state.tera.render("node_view.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    } else {
        (axum::http::StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}

// Helper for file tree
#[derive(Serialize)]
pub struct FileTreeItem {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String, // "directory" or "file"
    pub children: Vec<FileTreeItem>,
}

pub fn get_directory_tree(path: &std::path::Path) -> FileTreeItem {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut children = Vec::new();

    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if file_name.starts_with('.') {
                    continue;
                }

                children.push(get_directory_tree(&path));
            }
        }
    }

    // Sort directories first
    children.sort_by(|a, b| {
        if a.item_type == b.item_type {
            a.name.cmp(&b.name)
        } else if a.item_type == "directory" {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    FileTreeItem {
        name,
        item_type: if path.is_dir() {
            "directory".to_string()
        } else {
            "file".to_string()
        },
        children,
    }
}
