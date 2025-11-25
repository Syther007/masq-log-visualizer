use masq_log_visualizer::parser::scan_directory;
use std::path::PathBuf;

#[test]
fn test_scan_flat_directory_structure() {
    let test_dir = PathBuf::from("./Example-Log-2");

    if !test_dir.exists() {
        eprintln!("Warning: Example-Log-2 directory not found, skipping test");
        return;
    }

    let result = scan_directory(&test_dir);
    assert!(
        result.is_ok(),
        "Failed to scan directory: {:?}",
        result.err()
    );

    let nodes = result.unwrap();
    assert!(
        !nodes.is_empty(),
        "No nodes found in Example-Log-2 directory"
    );

    // Check for expected nodes "1", "2", "3", "4", "5", "6"
    let expected_nodes = vec!["1", "2", "3", "4", "5", "6"];
    for expected in &expected_nodes {
        assert!(
            nodes.contains_key(*expected),
            "Node {} not found in parsed results",
            expected
        );

        let node = nodes.get(*expected).unwrap();
        assert_eq!(node.name, *expected, "Node name mismatch");

        // Verify log file association
        assert!(
            !node.log_files.is_empty(),
            "Node {} should have log files",
            expected
        );
        let log_file = &node.log_files[0];
        assert!(
            log_file.contains("MASQNode_rCURRENT.log"),
            "Log file name incorrect: {}",
            log_file
        );

        // Verify database association
        // Note: In flat structure, we might need to verify how DB paths are stored
        // For now, just check that the parser found *something* if it's supposed to
    }

    println!(
        "Successfully parsed {} nodes from flat structure",
        nodes.len()
    );
}
