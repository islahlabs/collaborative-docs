mod app;
mod auth;
mod config;
mod crdt;
mod database;
mod error;
mod handlers;
mod models;
mod openapi;
mod tests;
mod utils;
mod websocket;

use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::{
    app::create_app,
    config::AppConfig,
    database::Database,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration
    let config = AppConfig::load()?;
    
    // Initialize database
    let database = Database::new(&config.database_url()).await.map_err(|e| {
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    })?;
    
    // Create application
    let app = create_app(database, &config);

    // Parse host address
    let host_ip = if config.server.host == "0.0.0.0" {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    } else {
        config.server.host.parse().unwrap_or_else(|_| {
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        })
    };
    
    let addr = SocketAddr::from((host_ip, config.server.port));
    
    info!("🚀 Server starting on http://{}", addr);
    info!("📝 API endpoints:");
    info!("  POST   /api/auth/signup");
    info!("  POST   /api/auth/login");
    info!("  POST   /api/doc (requires authentication)");
    info!("  PUT    /api/admin/users/{{user_id}}/role (admin only)");
    info!("  GET    /api/doc/{{id}}");
    info!("  PUT    /api/doc/{{id}}");
    info!("  GET    /api/doc/{{id}}/history");
    info!("  GET    /api/doc/{{id}}/stats");
    info!("  GET    /api/search?q=query");
    info!("  GET    /api/doc/{{id}}/crdt/state");
    info!("  POST   /api/doc/{{id}}/crdt/update");
    info!("  GET    /ws/doc/{{document_id}} (WebSocket)");
    info!("  GET    /ws/info/{{document_id}}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
