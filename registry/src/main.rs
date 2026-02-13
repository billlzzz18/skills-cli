mod api;
mod mcp;
mod types;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Define routes
    let app = Router::new()
        .route("/api/v1/search", get(api::search_skills))
        .route("/api/v1/skills/:namespace/:name", get(api::get_skill))
        .route("/mcp/sse", get(mcp::mcp_sse_handler))
        .layer(TraceLayer::new_for_http());

    // Run server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("SkillHub Registry listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
