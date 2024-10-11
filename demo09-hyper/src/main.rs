use axum::{response::Html, routing::get, Router};


#[tokio::main]
async fn main () {
    let app = Router::new()
        .route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    println!("Listening on 127.0.0.1:3001");
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>hello wowrld</h1>")
}