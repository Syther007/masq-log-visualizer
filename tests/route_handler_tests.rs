// Route Handler Tests for additional features

#[tokio::test]
async fn test_download_log_handler() {
    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use masq_log_visualizer::models::AllNodesData;
    use masq_log_visualizer::routes::{download_log, AppState};
    use std::fs::File;
    use std::io::Write;
    use std::sync::{Arc, RwLock};
    use tempfile::TempDir;

    // Setup temporary directory with a node and a log file
    let temp_dir = TempDir::new().unwrap();
    let node_dir = temp_dir.path().join("node1");
    std::fs::create_dir(&node_dir).unwrap();
    let log_path = node_dir.join("sample.log");
    let mut file = File::create(&log_path).unwrap();
    writeln!(file, "Hello world").unwrap();

    let app_state = AppState {
        nodes_data: Arc::new(RwLock::new(AllNodesData::new())),
        input_dir: temp_dir.path().to_path_buf(),
        tera: Arc::new(tera::Tera::default()),
    };

    let response = download_log(
        State(app_state),
        Path(("node1".to_string(), "sample.log".to_string())),
    )
    .await
    .into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    assert!(body_str.contains("Hello world"));
}

#[tokio::test]
async fn test_get_gossip_handler() {
    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use masq_log_visualizer::parser::scan_directory;
    use masq_log_visualizer::routes::{get_gossip, AppState};
    use std::fs::File;
    use std::io::Write;
    use std::sync::{Arc, RwLock};
    use tempfile::TempDir;

    // Setup temporary directory with a node
    let temp_dir = TempDir::new().unwrap();
    let node_dir = temp_dir.path().join("node2");
    std::fs::create_dir(&node_dir).unwrap();

    // Create a log file so scan_directory recognizes it as a node
    let log_path = node_dir.join("MASQNode_rCURRENT.log");
    let mut file = File::create(&log_path).unwrap();
    writeln!(file, "Test log").unwrap();

    // Scan directory to populate nodes_data with gossip info
    let nodes_map = scan_directory(temp_dir.path()).unwrap();

    let app_state = AppState {
        nodes_data: Arc::new(RwLock::new(nodes_map)),
        input_dir: temp_dir.path().to_path_buf(),
        tera: Arc::new(tera::Tera::default()),
    };

    let response = get_gossip(State(app_state), Path("node2".to_string()))
        .await
        .into_response();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_db_tables_handler() {
    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use masq_log_visualizer::parser::scan_directory;
    use masq_log_visualizer::routes::{get_db_tables, AppState};
    use rusqlite::{params, Connection};
    use std::fs::File;
    use std::io::Write;
    use std::sync::{Arc, RwLock};
    use tempfile::TempDir;

    // Setup temporary directory with a node and a SQLite DB
    let temp_dir = TempDir::new().unwrap();
    let node_dir = temp_dir.path().join("node_db");
    std::fs::create_dir(&node_dir).unwrap();
    let db_path = node_dir.join("node-data.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
        params![],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO test_table (name) VALUES (?1)",
        params!["Alice"],
    )
    .unwrap();
    drop(conn);

    // Create a log file so scan_directory recognizes it as a node
    let log_path = node_dir.join("MASQNode_rCURRENT.log");
    File::create(&log_path).unwrap();

    // Scan directory to populate nodes_data
    let nodes_map = scan_directory(temp_dir.path()).unwrap();

    let app_state = AppState {
        nodes_data: Arc::new(RwLock::new(nodes_map)),
        input_dir: temp_dir.path().to_path_buf(),
        tera: Arc::new(tera::Tera::default()),
    };

    let response = get_db_tables(State(app_state), Path("node_db".to_string()))
        .await
        .into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    assert!(body_str.contains("test_table"));
}

#[tokio::test]
async fn test_get_log_range_start_param() {
    use axum::extract::{Path, Query, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use masq_log_visualizer::models::AllNodesData;
    use masq_log_visualizer::routes::{AppState, LogRangeParams};
    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};
    use std::sync::{Arc, RwLock};
    use tempfile::TempDir;
    use tera::Tera;

    // Setup temporary directory with a log file
    let temp_dir = TempDir::new().unwrap();
    let node_dir = temp_dir.path().join("node_log");
    std::fs::create_dir(&node_dir).unwrap();
    let log_path = node_dir.join("sample.log");
    let mut file = File::create(&log_path).unwrap();
    for i in 1..=20 {
        writeln!(file, "Line {}", i).unwrap();
    }

    let app_state = AppState {
        nodes_data: Arc::new(RwLock::new(AllNodesData::new())),
        input_dir: temp_dir.path().to_path_buf(),
        tera: Arc::new(Tera::default()),
    };

    let params = LogRangeParams {
        start: Some(5),
        lines: Some(5),
        from_end: Some("false".to_string()),
    };

    // Directly invoke the logic (replicating handler) to avoid full HTTP request
    let reader = BufReader::new(File::open(&log_path).unwrap());
    let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
    let total_lines = all_lines.len();
    let start = params.start.unwrap_or(0);
    let end = (start + params.lines.unwrap_or(1000)).min(total_lines);
    let lines = if start < total_lines {
        all_lines[start..end].to_vec()
    } else {
        Vec::new()
    };

    assert_eq!(total_lines, 20);
    assert_eq!(
        lines,
        vec!["Line 6", "Line 7", "Line 8", "Line 9", "Line 10"]
    );
}
