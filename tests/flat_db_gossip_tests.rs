// TDD tests for database and gossip with flat structure
use masq_log_visualizer::parser::scan_directory;
use masq_log_visualizer::routes::AppState;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use axum::Router;
use axum::routing::get;
use tower::ServiceExt;
use axum::http::{Request, StatusCode};
use axum::body::Body;

#[tokio::test]
async fn test_flat_structure_database_api() {
    let test_dir = PathBuf::from("./Example-Log-2");
    
    if !test_dir.exists() {
        eprintln!("Skipping: Example-Log-2 not found");
        return;
    }
    
    let nodes = scan_directory(&test_dir).expect("Failed to scan");
    
    let tera = tera::Tera::new("templates/**/*").expect("Failed to load templates");
    
    let app_state = AppState {
        nodes_data: Arc::new(RwLock::new(nodes)),
        input_dir: test_dir,
        tera: Arc::new(tera),
    };
    
    let app = Router::new()
        .route("/api/db/:node_name", get(masq_log_visualizer::routes::get_db_tables))
        .route("/api/db/:node_name/:table_name", get(masq_log_visualizer::routes::get_db_table_data))
        .with_state(app_state);
    
    // Test getting database tables list
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/db/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK, 
        "Should successfully get database tables for flat structure node");
    
    // Test getting specific table data (if node has a config table)
    let response2 = app
        .oneshot(
            Request::builder()
                .uri("/api/db/1/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Should either be OK or 404 if table doesn't exist (but not 500)
    assert!(response2.status() == StatusCode::OK || response2.status() == StatusCode::NOT_FOUND,
        "Database table request should return OK or NOT_FOUND, got: {}", response2.status());
}

#[tokio::test]
async fn test_flat_structure_gossip_parsing() {
    let test_dir = PathBuf::from("./Example-Log-2");
    
    if !test_dir.exists() {
        eprintln!("Skipping: Example-Log-2 not found");
        return;
    }
    
    let nodes = scan_directory(&test_dir).expect("Failed to scan");
    
    // Check if any node has gossip data
    let has_gossip = nodes.values().any(|n| !n.gossip.is_empty());
    
    if !has_gossip {
        println!("No gossip data in Example-Log-2 logs - this is expected if logs don't contain gossip");
        return;
    }
    
    // If we have gossip, test the API
    let tera = tera::Tera::new("templates/**/*").expect("Failed to load templates");
    
    let app_state = AppState {
        nodes_data: Arc::new(RwLock::new(nodes)),
        input_dir: test_dir,
        tera: Arc::new(tera),
    };
    
    let app = Router::new()
        .route("/api/gossip/:node_name", get(masq_log_visualizer::routes::get_gossip))
        .with_state(app_state);
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/gossip/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK,
        "Should successfully get gossip data for flat structure node");
}
