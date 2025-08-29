use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum::{response::Html, routing::get, Router};

use axum::extract::Path;
use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode};
use axum::extract::State;
use tower_http::services::ServeDir;
use std::sync::{Arc, Mutex};

struct MyCounter {
    counter: AtomicUsize,
}

struct MyConfig {
    text: String
}

struct MyState(i32);

/**
 * nest嵌套路由, 带状态
 */
fn service_one() -> Router {
    let state = Arc::new(MyState(5));
    Router::new()
        .route("/", get(sv1_handler))
        .with_state(state)
}

async fn sv1_handler(Extension(counter): Extension<Arc<MyCounter>>, State(state): State<Arc<MyState>>) -> Html<String> {
    counter.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Html(format!("Service {}-{}", counter.counter.load(std::sync::atomic::Ordering::Relaxed), state.0))
}
fn service_two() -> Router {
    Router::new().route("/", get(|| async {Html("Service Two".to_string())}))
}


#[tokio::main]
async fn main() {
    let shared_counter = Arc::new(MyCounter {
        counter: AtomicUsize::new(0)
    });

    let shared_text = Arc::new(MyConfig {
        text: "This is my counfigration".to_string()
    });

    let app = Router::new()
        .nest("/1", service_one())
        .nest("/2", service_two())
        .route("/", get(handler))
        .route("/book/:id", get(path_extract))
        .route("/book", get(query_extract))
        .route("/header", get(header_extract))
        .layer(Extension(shared_counter))
        .layer(Extension(shared_text))
        .fallback_service(ServeDir::new("web"));
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    println!("Listening on 127.0.0.1:3001");
    axum::serve(listener, app).await.unwrap();
}


/**
 * Arc, Extension
 */
async fn handler(Extension(counter): Extension<Arc<MyCounter>>, Extension(config): Extension<Arc<MyConfig>>) -> Html<String>{
    counter.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Html(format!("<h2>{} You are visitor number {}</h2>", config.text, counter.counter.load(std::sync::atomic::Ordering::Relaxed)))
}

/**
 * 2.3.1. Path Extract
 * http://127.0.0.1:3001/book/12
 */
async fn path_extract(Path(id) : Path<u32>) -> Html<String> {
    Html(format!("Hello, {id}!"))
}

/**
 * 2.3.2. Query Extract
 * http://127.0.0.1:3001/book?id=12&foo=bar&blah=hello
 */
async fn query_extract(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    Html(format!("Hello, {params:#?}"))
}

/**
 * 2.3.3. Header Extract
 * http://127.0.0.1:3001/
 */
async fn header_extract(headers: HeaderMap) -> Html<String> {
    Html(format!("{headers:#?}"))
}


/**
 * 返回状态码
 */
async fn return_status_code() -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = std::time::SystemTime::now();
    let seconds_wrapped = start
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Bad clock".to_string()))?
        .as_secs() % 3;
    let divided = 100u64.checked_div(seconds_wrapped)
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "div by 0".to_string()))?;

    Ok(Json(divided))
}



