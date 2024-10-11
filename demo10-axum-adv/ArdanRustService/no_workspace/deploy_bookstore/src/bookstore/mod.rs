mod configuration;
mod db;
mod web_service;
use anyhow::Result;
use axum::{middleware, routing::{get, post}, Extension, Router};
use tower_http::cors::CorsLayer;
use crate::auth::auth_layers;
use crate::auth::auth_layers::ListenPort;

pub async fn setup_service(listen_port: String) -> Result<Router> {
    let config = configuration::BookstoreConfiguration::load()?;
    let db_pool = db::get_connection_pool(&config.db_filename).await?;

    db::perform_migrations(db_pool.clone()).await?;

    let secure_router = Router::new()
        .layer(CorsLayer::very_permissive())
        .layer(Extension(config.clone()))
        .layer(Extension(db_pool.clone()))
        .route("/add", post(web_service::add_book))
        .route("/delete/:id", get(web_service::delete_book))
        .route("/update/:id", post(web_service::update_book))
        .route_layer(middleware::from_fn(auth_layers::require_remote_token))
        .layer(Extension(ListenPort(listen_port)))
        ;

    let router = Router::new()
        .merge(secure_router)
        .route("/", get(web_service::all_books))
        .route("/:id", get(web_service::get_book))
        .layer(Extension(config))
        .layer(Extension(db_pool));

    Ok(router)
}
