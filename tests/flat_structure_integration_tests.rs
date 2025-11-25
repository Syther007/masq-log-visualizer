// Integration tests for flat directory structure file access

use masq_log_visualizer::parser::scan_directory;
use masq_log_visualizer::routes::AppState;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use axum::Router;
use axum::routing::get;
use tower::ServiceExt;
use axum::http::{Request, StatusCode};
use axum::body::Body;

fn setup_flat_test_app() -> Router {
    let test_dir = PathBuf::from("./Example-Log-2");
    
    if !test_dir.exists() {
        panic!("Example-Log-2 directory not found");
    }
    
    let nodes = scan_directory(&test_dir).expect("Failed to scan directory");
    
    let app_state = AppState {
        nodes_data: Arc::new(RwLock::new(nodes)),
        input_dir: test_dir,
        tera: Arc::new(tera::Tera::default()),
    };
    
    Router::new()
        .route("/api/logs/:node/:file/range", get(masq_log_visualizer::routes::get_log_range))
        .route("/api/download/:node/:file", get(masq_log_visualizer::routes::download_log))
        .route("/api/db/:node", get(masq_log_visualizer::routes::get_db_tables))
        .route("/api/gossip/:node", get(masq_log_visualizer::routes::get_gossip))
        .with_state(app_state)
}

#[tokio::test]
async fn test_flat_structure_log_access() {
    let test_dir = PathBuf::from("./Example-Log-2");
    if !test_dir.exists() {
        eprintln!("Skipping test: Example-Log-2 not found");
        return;
    }
    
    let app = setup_flat_test_app();
    
    // Try to access a log file from node "1"
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/logs/1/1-MASQNode_rCURRENT.log/range?lines=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // This should succeed (200 OK) but will currently fail
    assert_eq!(response.status(), StatusCode::OK, 
        "Failed to access log from flat structure");
}

#[tokio::test]
async fn test_flat_structure_download_log() {
    let test_dir = PathBuf::from("./Example-Log-2");
    if !test_dir.exists() {
        eprintln!("Skipping test: Example-Log-2 not found");
        return;
    }
    
    let app = setup_flat_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/download/1/1-MASQNode_rCURRENT.log")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK,
        "Failed to download log from flat structure");
}

#[tokio::test]
async fn test_flat_structure_db_access() {
    let test_dir = PathBuf::from("./Example-Log-2");
    if !test_dir.exists() {
        eprintln!("Skipping test: Example-Log-2 not found");
        return;
    }
    
    let app = setup_flat_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/db/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK,
        "Failed to access database from flat structure");
}
