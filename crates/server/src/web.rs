use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../../packages/solid-web/dist"]
struct Asset;

pub struct StaticFile(pub String);

impl IntoResponse for StaticFile {
    fn into_response(self) -> Response {
        let path = self.0;

        let mut mime_type = mime_guess::from_path(&path).first_or_octet_stream();
        let asset = Asset::get(&path).or_else(|| {
            mime_type = mime::TEXT_HTML;
            Asset::get("index.html")
        });

        match asset {
            Some(content) => {
                let m = mime_guess::from_path(path).first_or(mime::TEXT_HTML);
                ([(header::CONTENT_TYPE, m.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_owned();

    StaticFile(path)
}
