use bytes::Bytes;
use http::{Request, Response, response::Parts};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use url::Url;

#[async_trait::async_trait]
pub trait HttpClient: Clone + Send + Sync + 'static {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Incoming>, Error>;

    async fn get(&self, url: &Url) -> Result<(Parts, Bytes), Error> {
        let resp = self
            .send(Request::get(url.as_str()).body(Default::default()).unwrap())
            .await?;
        let (parts, incoming) = resp.into_parts();
        let body = incoming.collect().await?.to_bytes();

        Ok((parts, body))
    }
}

#[derive(Debug, Clone)]
pub struct HyperClient {
    client: hyper_util::client::legacy::Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
}

impl HyperClient {
    pub fn new(
        client: hyper_util::client::legacy::Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    ) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl HttpClient for HyperClient {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Incoming>, Error> {
        let (parts, body) = request.into_parts();

        let resp = self
            .client
            .request(Request::from_parts(parts, body.into()))
            .await?;

        Ok(resp)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Client(#[from] hyper_util::client::legacy::Error),

    #[error(transparent)]
    Http(#[from] hyper::Error),
}
