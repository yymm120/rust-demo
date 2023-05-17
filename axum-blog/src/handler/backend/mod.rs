use axum::Router;
use axum::routing::get;
use crate::handler::backend::index::index;

mod index;


pub fn router() -> Router {
    Router::new().route("/", get(index))
}