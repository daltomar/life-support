mod handlers;

use axum::{Router, routing::get};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let media_dir = std::env::var("MEDIA_DIR").unwrap_or_else(|_| "media".to_string());

    let app = Router::new()
        .route("/", get(handlers::player))
        .nest_service("/media", ServeDir::new(&media_dir));

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind BIND_ADDR");

    println!("listening on {bind_addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
}
