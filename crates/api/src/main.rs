use std::error::Error;

use axum::{http::StatusCode, routing, Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new().route("/", routing::get(hello_world));

    let listener = TcpListener::bind("localhost:3001").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[axum::debug_handler]
async fn hello_world() -> (StatusCode, Json<Message>) {
    let msg = Message {
        value: String::from("Hello world!"),
    };

    (StatusCode::CREATED, Json(msg))
}

#[derive(Serialize)]
struct Message {
    value: String,
}
