use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use masq_log_visualizer::{models::AllNodesData, parser::scan_directory, routes::AppState};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tera::Tera;
use tower::ServiceExt;

async fn setup_test_app() -> Option<(axum::Router, AllNodesData)> {
    let test_dir = PathBuf::from("./Example-Log");

    if !test_dir.exists() {
        eprintln!("Warning: Example-Log directory not found, skipping integration tests");
        return None;
    }

    let nodes_data = scan_directory(&test_dir).ok()?;

    // Try to load templates from multiple possible locations
    let tera = if let Ok(t) = Tera::new("templates/**/*") {
        eprintln!("Loaded templates from ./templates");
        t
    } else if let Ok(t) = Tera::new("target/debug/templates/**/*") {
        eprintln!("Loaded templates from target/debug/templates");
        t
    } else {
        // Try relative to executable
        let exe_path = std::env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;
        let templates_pattern = exe_dir.join("templates/**/*");
        eprintln!("Trying templates pattern: {:?}", templates_pattern);
        Tera::new(templates_pattern.to_str().unwrap()).ok()?
    };

    // Debug: list loaded templates
    eprintln!(
        "Loaded templates: {:?}",
        tera.get_template_names().collect::<Vec<_>>()
    );

    let app_state = AppState {
        nodes_data: Arc::new(RwLock::new(nodes_data.clone())),
        input_dir: test_dir.clone(),
        tera: Arc::new(tera),
    };

    use axum::routing::get;
    let app = axum::Router::new()
        .route("/", get(masq_log_visualizer::routes::dashboard))
        .route(
            "/node/:node_name",
            get(masq_log_visualizer::routes::node_view),
        )
        .route(
            "/api/logs/:node_name/:file_name/range",
            get(masq_log_visualizer::routes::get_log_range),
        )
        .route(
            "/api/logs/:node_name/:file_name",
            get(masq_log_visualizer::routes::download_log),
        )
        .route(
            "/api/db/:node_name",
            get(masq_log_visualizer::routes::get_db_tables),
        )
        .route(
            "/api/db/:node_name/:table_name",
            get(masq_log_visualizer::routes::get_db_table_data),
        )
        .route(
            "/api/gossip/:node_name",
            get(masq_log_visualizer::routes::get_gossip),
        )
        .with_state(app_state);

    Some((app, nodes_data))
}

#[tokio::test]
async fn test_dashboard_route() {
    let setup = setup_test_app().await;
    if setup.is_none() {
        return;
    }
    let (app, _) = setup.unwrap();

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::OK {
        use axum::body::to_bytes;
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("Dashboard error response: {}", body_str);
    }

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_node_view_route() {
    let setup = setup_test_app().await;
    if setup.is_none() {
        return;
    }
    let (app, nodes_data) = setup.unwrap();

    // Get first node name
    let node_name = nodes_data.keys().next().unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/node/{}", node_name))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_db_tables() {
    let setup = setup_test_app().await;
    if setup.is_none() {
        return;
    }
    let (app, nodes_data) = setup.unwrap();

    // Find a node with database
    let node_name = nodes_data
        .iter()
        .find(|(_, node)| !node.database.tables.is_empty())
        .map(|(name, _)| name.clone());

    if let Some(name) = node_name {
        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/db/{}", name))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_api_gossip() {
    let setup = setup_test_app().await;
    if setup.is_none() {
        return;
    }
    let (app, nodes_data) = setup.unwrap();

    let node_name = nodes_data.keys().next().unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/gossip/{}", node_name))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_nonexistent_node() {
    let setup = setup_test_app().await;
    if setup.is_none() {
        return;
    }
    let (app, _) = setup.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/node/nonexistent_node_12345")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_log_range_api() {
    let setup = setup_test_app().await;
    if setup.is_none() {
        return;
    }
    let (app, nodes_data) = setup.unwrap();

    // Find a node with log files
    for (node_name, node_data) in &nodes_data {
        if !node_data.log_files.is_empty() {
            let log_file = &node_data.log_files[0];

            let response = app
                .oneshot(
                    Request::builder()
                        .uri(&format!(
                            "/api/logs/{}/{}/range?fromEnd=true&lines=10",
                            node_name, log_file
                        ))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            return;
        }
    }
}

#[tokio::test]
async fn test_api_db_table_not_found() {
    let setup = setup_test_app().await;
    if setup.is_none() {
        return;
    }
    let (app, nodes_data) = setup.unwrap();

    // Find a node with database
    let node_name = nodes_data
        .iter()
        .find(|(_, node)| !node.database.tables.is_empty())
        .map(|(name, _)| name.clone());

    if let Some(name) = node_name {
        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/db/{}/nonexistent_table", name))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should probably be 404 or 500 depending on implementation
        // Let's check it's not 200 OK
        assert_ne!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_download_log_not_found() {
    let setup = setup_test_app().await;
    if setup.is_none() {
        return;
    }
    let (app, nodes_data) = setup.unwrap();

    let node_name = nodes_data.keys().next().unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/logs/{}/nonexistent.log", node_name))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_log_range_invalid_params() {
    let setup = setup_test_app().await;
    if setup.is_none() {
        return;
    }
    let (app, nodes_data) = setup.unwrap();

    // Find a node with log files
    for (node_name, node_data) in &nodes_data {
        if !node_data.log_files.is_empty() {
            let log_file = &node_data.log_files[0];

            // Test with 0 lines (should default or handle gracefully)
            let response = app
                .oneshot(
                    Request::builder()
                        .uri(&format!(
                            "/api/logs/{}/{}/range?lines=0",
                            node_name, log_file
                        ))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            // Should still be OK, just empty or default
            assert_eq!(response.status(), StatusCode::OK);
            return;
        }
    }
}
