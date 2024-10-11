//! Run with
//!
//! ```not_rust
//! cargo run -p example-static-file-server
//! ```

use axum::{
    extract::Request, handler::HandlerWithoutStateExt, http::StatusCode, routing::get, Router,
};
use std::net::SocketAddr;
use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_static_file_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tokio::join!(
        serve(using_serve_dir(), 3001),
    );
}

fn using_serve_dir() -> Router {
    // serve the file in the "assets" directory under `/assets`
    Router::new().nest_service("/", ServeDir::new("site"))
}


async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}
// use tracing::{info, instrument};
// use tracing_subscriber::fmt::format::FmtSpan;

// #[tokio::main]
// async fn main() {
//     let subscriber = tracing_subscriber::fmt()
//         .with_span_events(FmtSpan::CLOSE)
//         .finish();

//     tracing::subscriber::set_global_default(subscriber).unwrap();
//     test_time();
//     info!("hello world");
// }

// #[instrument]
// fn test_time() {
//     info!("test time");
// }
