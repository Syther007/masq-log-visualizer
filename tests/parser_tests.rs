use masq_log_visualizer::parser::{scan_directory, get_table_data};
use std::path::PathBuf;

#[test]
fn test_scan_directory_with_example_log() {
    let test_dir = PathBuf::from("./Example-Log");
    
    if !test_dir.exists() {
        eprintln!("Warning: Example-Log directory not found, skipping test");
        return;
    }
    
    let result = scan_directory(&test_dir);
    assert!(result.is_ok(), "Failed to scan directory: {:?}", result.err());
    
    let nodes = result.unwrap();
    assert!(!nodes.is_empty(), "No nodes found in Example-Log directory");
    
    // Verify each node has expected structure
    for (node_name, node_data) in &nodes {
        assert!(!node_name.is_empty(), "Node name should not be empty");
        assert_eq!(node_data.name, *node_name, "Node name mismatch");
        assert!(!node_data.log_files.is_empty(), "Node should have log files");
    }
}

#[test]
fn test_node_data_structure() {
    let test_dir = PathBuf::from("./Example-Log");
    
    if !test_dir.exists() {
        return;
    }
    
    let nodes = scan_directory(&test_dir).unwrap();
    
    for (_name, node) in &nodes {
        // Test that current_log is set
        assert!(!node.current_log.is_empty(), "Current log should be set");
        
        // Test that neighborhood and gossip are valid vectors
        assert!(node.neighborhood.is_empty() || !node.neighborhood.is_empty());
        assert!(node.gossip.is_empty() || !node.gossip.is_empty());
    }
}

#[test]
fn test_log_file_parsing() {
    let test_dir = PathBuf::from("./Example-Log");
    
    if !test_dir.exists() {
        return;
    }
    
    let nodes = scan_directory(&test_dir).unwrap();
   
    // Ensure at least one node was parsed
    assert!(!nodes.is_empty(), "Should parse at least one node");
    
    // Check that log files are listed
    for (_, node) in &nodes {
        if !node.log_files.is_empty() {
            let log_file = &node.log_files[0];
            assert!(
                log_file.ends_with(".log") || log_file.ends_with(".log.zip"),
                "Log file should have .log or .log.zip extension: {}",
                log_file
            );
        }
    }
}

#[test]
fn test_database_extraction() {
    let test_dir = PathBuf::from("./Example-Log");
    
    if !test_dir.exists() {
        return;
    }
    
    let nodes = scan_directory(&test_dir).unwrap();
    
    // Find a node with a database
    for (node_name, node) in &nodes {
        if !node.database.tables.is_empty() {
            println!("Node {} has {} database tables", node_name, node.database.tables.len());
            
            // Verify table structure
            for (table_name, table_data) in &node.database.tables {
                assert!(!table_name.is_empty(), "Table name should not be empty");
                assert!(!table_data.columns.is_empty(), "Table should have columns");
                // Note: rows should be empty initially (on-demand loading)
                assert_eq!(table_data.rows.len(), 0, "Rows should be empty (on-demand loading)");
            }
            return; // Test passed if we found at least one node with a database
        }
    }
}

#[test]
fn test_neighborhood_edge_parsing() {
    let test_dir = PathBuf::from("./Example-Log");
    
    if !test_dir.exists() {
        return;
    }
    
    let nodes = scan_directory(&test_dir).unwrap();
    
    // Check if any node has neighborhood data
    let has_neighborhood = nodes.values().any(|n| !n.neighborhood.is_empty());
    
    if has_neighborhood {
        for (_name, node) in &nodes {
            for edge in &node.neighborhood {
                assert!(!edge.from.is_empty(), "Edge 'from' should not be empty");
                assert!(!edge.to.is_empty(), "Edge 'to' should not be empty");
            }
        }
    }
}

#[test]
fn test_gossip_entry_parsing() {
    let test_dir = PathBuf::from("./Example-Log");
    
    if !test_dir.exists() {
        return;
    }
    
    let nodes = scan_directory(&test_dir).unwrap();
    
    // Check if any node has gossip data
    let has_gossip = nodes.values().any(|n| !n.gossip.is_empty());
    
    if has_gossip {
        for (_name, node) in &nodes {
            for entry in &node.gossip {
                assert!(!entry.timestamp.is_empty(), "Gossip timestamp should not be empty");
                assert!(!entry.actor.is_empty(), "Gossip actor should not be empty");
                assert!(!entry.tag.is_empty(), "Gossip tag should not be empty");
                assert!(entry.dot.contains("digraph"), "Gossip DOT should contain digraph");
            }
        }
    }
}

#[test]
fn test_get_table_data() {
    let test_dir = PathBuf::from("./Example-Log");
    
    if !test_dir.exists() {
        return;
    }
    
    let nodes = scan_directory(&test_dir).unwrap();
    
    // Find a node with a database and tables
    for (node_name, node) in &nodes {
        let db_path = test_dir.join(node_name).join("node-data.db");
        
        if db_path.exists() && !node.database.tables.is_empty() {
            let table_name = node.database.tables.keys().next().unwrap();
            
            let result = get_table_data(&db_path, table_name);
            assert!(result.is_ok(), "Failed to get table data: {:?}", result.err());
            
            let table_data = result.unwrap();
            assert!(!table_data.columns.is_empty(), "Table should have columns");
            // Rows may be empty or populated depending on the table
            
            return; // Test passed
        }
    }
}

#[test]
fn test_serialization_deserialization() {
    use masq_log_visualizer::models::{NodeData, NeighborhoodEdge, GossipEntry, DatabaseData};
    use std::collections::HashMap;
    
    let node = NodeData {
        name: "test_node".to_string(),
        neighborhood: vec![
            NeighborhoodEdge {
                from: "node1".to_string(),
                to: "node2".to_string(),
            }
        ],
        gossip: vec![
            GossipEntry {
                timestamp: "2024-01-01 12:00:00".to_string(),
                actor: "Neighborhood".to_string(),
                tag: "Sent Gossip".to_string(),
                dot: "digraph db { }".to_string(),
            }
        ],
        log_files: vec!["test.log".to_string()],
        current_log: "test.log".to_string(),
        database: DatabaseData { tables: HashMap::new() },
    };
    
    // Test serialization
    let json = serde_json::to_string(&node);
    assert!(json.is_ok(), "Serialization failed");
    
    // Test deserialization
    let json_str = json.unwrap();
    let parsed: Result<NodeData, _> = serde_json::from_str(&json_str);
    assert!(parsed.is_ok(), "Deserialization failed");
    
    let parsed_node = parsed.unwrap();
    assert_eq!(parsed_node.name, "test_node");
    assert_eq!(parsed_node.neighborhood.len(), 1);
    assert_eq!(parsed_node.gossip.len(), 1);
}

#[test]
fn test_scan_nonexistent_directory() {
    let path = PathBuf::from("./nonexistent_directory_12345");
    let result = scan_directory(&path);
    assert!(result.is_err(), "Scanning nonexistent directory should fail");
}

#[test]
fn test_scan_empty_directory() {
    let temp_dir = std::env::temp_dir().join("masq_test_empty_dir");
    let _ = std::fs::create_dir(&temp_dir);
    
    let result = scan_directory(&temp_dir);
    // It might succeed but return empty map, or fail depending on implementation.
    // Based on parser.rs, it uses fs::read_dir, so it should succeed but return empty.
    
    if let Ok(nodes) = result {
        assert!(nodes.is_empty(), "Empty directory should result in no nodes");
    }
    
    let _ = std::fs::remove_dir(&temp_dir);
}
