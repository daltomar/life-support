mod handlers;

use axum::{routing::get, Router};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let media_dir = std::env::var("MEDIA_DIR").unwrap_or_else(|_| "media".to_string());
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string());

    if !std::path::Path::new(&media_dir).exists() {
        eprintln!("warning: MEDIA_DIR '{media_dir}' does not exist");
    }
    if !std::path::Path::new(&static_dir).exists() {
        eprintln!("warning: STATIC_DIR '{static_dir}' does not exist");
    }

    let app = Router::new()
        .route("/", get(handlers::releases))
        .route("/releases", get(handlers::releases))
        .nest_service("/media", ServeDir::new(&media_dir))
        .nest_service("/static", ServeDir::new(&static_dir));

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
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = sigterm.recv() => {}
    }
}
