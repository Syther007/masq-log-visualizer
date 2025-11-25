// End-to-end test to verify template gets correct currentLogFile
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use masq_log_visualizer::parser::scan_directory;
use masq_log_visualizer::routes::AppState;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tower::ServiceExt;

#[tokio::test]
async fn test_flat_structure_template_rendering() {
    let test_dir = PathBuf::from("./Example-Log-2");

    if !test_dir.exists() {
        eprintln!("Skipping: Example-Log-2 not found");
        return;
    }

    let nodes = scan_directory(&test_dir).expect("Failed to scan");

    // Load templates
    let tera = tera::Tera::new("templates/**/*").expect("Failed to load templates");

    let app_state = AppState {
        nodes_data: Arc::new(RwLock::new(nodes)),
        input_dir: test_dir,
        tera: Arc::new(tera),
    };

    let app = Router::new()
        .route(
            "/node/:node_name",
            get(masq_log_visualizer::routes::node_view),
        )
        .with_state(app_state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/node/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should render page successfully"
    );

    // Get the HTML body
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();

    // Check that the HTML contains the correct JavaScript variable
    print!("Checking HTML for currentLogFile...\n");

    // Should contain: const currentLogFile = "1-MASQNode_rCURRENT.log";
    assert!(
        body_str.contains("const currentLogFile ="),
        "HTML should contain currentLogFile variable declaration"
    );

    assert!(
        body_str.contains("1-MASQNode_rCURRENT.log"),
        "HTML should contain the correct filename with prefix:\n{}",
        body_str
            .lines()
            .find(|line| line.contains("currentLogFile"))
            .unwrap_or("LINE NOT FOUND")
    );

    // Should NOT contain the old hardcoded value in the fetch calls
    let fetch_lines: Vec<&str> = body_str
        .lines()
        .filter(|line| line.contains("/api/logs/") && line.contains("/range"))
        .collect();

    for line in &fetch_lines {
        assert!(
            line.contains("${currentLogFile}") || line.contains("${ currentLogFile}"),
            "API fetch should use currentLogFile variable, found: {}",
            line
        );
    }

    println!("✅ Template correctly renders currentLogFile variable!");
}
