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
    // We'll load templates from the "templates" directory relative to the executable or CWD
    // For development, we can assume "templates/**/*"
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    // Disable autoescape for HTML generation if needed, but usually safe to keep on
    // tera.autoescape_on(vec![]);

    let app_state = routes::AppState {
        nodes_data: Arc::new(RwLock::new(nodes_data)),
        input_dir: input_dir.clone(),
        tera: Arc::new(tera),
    };

    // Setup Router
    let app = Router::new()
        .route("/", get(routes::dashboard))
        .route("/node/:node_name", get(routes::node_view))
        .route("/api/logs/:node_name/:file_name/range", get(routes::get_log_range))
        .route("/api/logs/:node_name/:file_name", get(routes::download_log))
        .route("/api/db/:node_name", get(routes::get_db_tables))
        .route("/api/db/:node_name/:table_name", get(routes::get_db_table_data))
        .route("/api/gossip/:node_name", get(routes::get_gossip))
        .nest_service("/assets", ServeDir::new("assets"))
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
