#![allow(unused)]

// -- start: mod 模块
mod config;
mod crypt;
mod ctx;
mod error;
mod log;
mod model;
mod web;

pub mod _dev_utils;

use std::net::SocketAddr;

pub use self::error::{Error, Result};
use axum::{middleware, Router};
use model::ModelManager;
use tower_cookies::CookieManagerLayer;
use tracing_subscriber::EnvFilter;

use crate::web::mw_auth::mw_ctx_resolve;
use crate::web::mw_res_map::mw_response_map;
use crate::web::{routes_login, routes_static};
pub use config::config;
use tracing::info;

// -- end: mod 模块

#[tokio::main]
async fn main() -> Result<()> {
    // Log Init
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("--> into main");
    // DB Init
    _dev_utils::init_dev().await;

    // Initialize ModelManager.
    let mm = ModelManager::new().await?;

    let routes_all = Router::new()
        .merge(routes_login::routes(mm.clone()))
        .layer(middleware::map_response(mw_response_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("{:<12} - {addr}\n", "LISTENING");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}
