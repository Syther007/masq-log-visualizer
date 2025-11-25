// Test to verify template context has correct currentLogFile
use masq_log_visualizer::parser::scan_directory;
use std::path::{Path, PathBuf};

#[test]
fn test_current_log_file_extraction() {
    // Test flat structure
    let test_dir = PathBuf::from("./Example-Log-2");

    if !test_dir.exists() {
        eprintln!("Skipping: Example-Log-2 not found");
        return;
    }

    let nodes = scan_directory(&test_dir).expect("Failed to scan");

    // For node "1", check what filename would be extracted
    if let Some(node) = nodes.get("1") {
        let current_log_file = node
            .log_files
            .first()
            .and_then(|path| {
                PathBuf::from(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "MASQNode_rCURRENT.log".to_string());

        println!("Extracted filename: {}", current_log_file);

        // Should include the prefix
        assert!(
            current_log_file.starts_with("1-"),
            "Expected filename to start with '1-', got: {}",
            current_log_file
        );
        assert!(
            current_log_file.contains("MASQNode_rCURRENT.log"),
            "Expected filename to contain 'MASQNode_rCURRENT.log', got: {}",
            current_log_file
        );
    } else {
        panic!("Node '1' not found");
    }

    // Test nested structure
    let test_dir2 = PathBuf::from("./Example-Log");
    if test_dir2.exists() {
        let nodes2 = scan_directory(&test_dir2).expect("Failed to scan nested");

        // Pick any node
        if let Some((_, node)) = nodes2.iter().next() {
            let current_log_file = node
                .log_files
                .first()
                .and_then(|path| {
                    PathBuf::from(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "MASQNode_rCURRENT.log".to_string());

            println!("Nested structure filename: {}", current_log_file);

            // Should NOT have a prefix (no hyphen at start)
            assert!(
                !current_log_file.starts_with(char::is_numeric),
                "Nested structure filename should not start with number: {}",
                current_log_file
            );
        }
    }
}
