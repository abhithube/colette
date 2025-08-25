use bytes::Bytes;
use http::{Request, Response};
use http_body_util::BodyExt;
use reqwest::{Body, Client, Url};

pub trait HttpClient: Send + Sync {
    fn send(
        &self,
        request: Request<Bytes>,
    ) -> impl Future<Output = Result<Response<Body>, Error>> + Send;

    fn get(&self, url: &Url) -> impl Future<Output = Result<Bytes, Error>> + Send {
        async {
            let resp = self
                .send(Request::get(url.as_str()).body(Default::default())?)
                .await?;

            let body = resp.into_body().collect().await?.to_bytes();

            Ok(body)
        }
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

    #[error(transparent)]
    Http(#[from] http::Error),
}
