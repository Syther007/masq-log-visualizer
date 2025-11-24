use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GossipEntry {
    pub timestamp: String,
    pub actor: String,
    pub tag: String,
    pub dot: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NeighborhoodEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DatabaseData {
    // For the full dump, we might want this. 
    // But for the optimized version, we might just store table names in NodeData 
    // and fetch content on demand. 
    // However, to match the original structure for the "data.js" generation:
    pub tables: HashMap<String, TableData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeData {
    pub name: String,
    pub neighborhood: Vec<NeighborhoodEdge>,
    pub gossip: Vec<GossipEntry>,
    #[serde(rename = "logFiles")]
    pub log_files: Vec<String>,
    #[serde(rename = "currentLog")]
    pub current_log: String,
    // In the Rust version, we might keep this empty or minimal until requested,
    // but for compatibility with the template rendering which expects `node.database.tables`,
    // we should include it.
    pub database: DatabaseData, 
}

pub type AllNodesData = HashMap<String, NodeData>;
