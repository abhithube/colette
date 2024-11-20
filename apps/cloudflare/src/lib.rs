use axum::{body::Body, http::Response, routing::get, Router};
use tower::Service;
use worker::{Context, Env, HttpRequest};

mod d1;
mod kv;

#[worker::event(fetch)]
async fn fetch(req: HttpRequest, _env: Env, _ctx: Context) -> worker::Result<Response<Body>> {
    console_error_panic_hook::set_once();

    let mut router = Router::new().route("/", get(root));
    let resp = router.call(req).await?;

    Ok(resp)
}

pub async fn root() -> &'static str {
    "Hello world!"
}
