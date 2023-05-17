use axum::Router;
use axum::routing::get;

mod index;


pub fn router() -> Router {
    Router::new().route("/", get(index::index))
}