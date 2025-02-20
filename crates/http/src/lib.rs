use bytes::Bytes;
use http::{Request, Response};
use http_body_util::BodyExt;
use reqwest::{Body, Client, Url};

#[async_trait::async_trait]
pub trait HttpClient: Send + Sync + 'static {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Body>, Error>;

    async fn get(&self, url: &Url) -> Result<Bytes, Error> {
        let resp = self
            .send(Request::get(url.as_str()).body(Default::default()).unwrap())
            .await?;

        let body = resp.into_body().collect().await?.to_bytes();

        Ok(body)
    }
}

#[derive(Debug, Clone)]
pub struct ReqwestClient {
    client: Client,
}

impl ReqwestClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl HttpClient for ReqwestClient {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Body>, Error> {
        let resp = self.client.execute(request.try_into()?).await?;

        Ok(resp.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Client(#[from] reqwest::Error),
}
