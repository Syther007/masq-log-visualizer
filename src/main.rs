mod models;
mod parser;
mod routes;

use axum::{
    routing::get,
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tera::Tera;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing node folders
    #[arg(short, long)]
    input: String,

    /// Server port
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let input_dir = PathBuf::from(&args.input);

    if !input_dir.exists() {
        eprintln!("Input directory does not exist: {:?}", input_dir);
        std::process::exit(1);
    }

    println!("Scanning input directory: {:?}", input_dir);
    let nodes_data = parser::scan_directory(&input_dir)?;
    println!("Found {} nodes.", nodes_data.len());

    // Initialize Tera templates
    // Find templates directory relative to executable location
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    
    // Try templates relative to exe first, then fall back to CWD (for dev)
    let templates_pattern = exe_dir.join("templates/**/*");
    let tera = match Tera::new(templates_pattern.to_str().unwrap()) {
        Ok(t) => t,
        Err(_) => {
            // Fallback to CWD for development
            match Tera::new("templates/**/*") {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Template parsing error: {}", e);
                    eprintln!("Tried paths: {:?} and ./templates/", templates_pattern);
                    std::process::exit(1);
                }
            }
        }
    };

    let app_state = routes::AppState {
        nodes_data: Arc::new(RwLock::new(nodes_data)),
        input_dir: input_dir.clone(),
        tera: Arc::new(tera),
    };

    // Setup Router
    // Determine assets directory (relative to exe or CWD)
    let assets_dir = exe_dir.join("assets");
    let assets_path = if assets_dir.exists() {
        assets_dir
    } else {
        PathBuf::from("assets") // Fallback to CWD for dev
    };
    
    let app = Router::new()
        .route("/", get(routes::dashboard))
        .route("/node/:node_name", get(routes::node_view))
        .route("/api/logs/:node_name/:file_name/range", get(routes::get_log_range))
        .route("/api/logs/:node_name/:file_name", get(routes::download_log))
        .route("/api/db/:node_name", get(routes::get_db_tables))
        .route("/api/db/:node_name/:table_name", get(routes::get_db_table_data))
        .route("/api/gossip/:node_name", get(routes::get_gossip))
        .nest_service("/assets", ServeDir::new(assets_path))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    println!("Server started successfully!");
    println!("URL: http://{}", addr);
    println!("Input directory: {:?}", input_dir);
    println!("Press Ctrl+C to stop the server.");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
