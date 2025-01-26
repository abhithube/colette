use bytes::Bytes;
use reqwest::{header::HeaderMap, Client as ReqwestClient, Method, RequestBuilder};

#[derive(Debug, Clone)]
pub struct Client {
    client: ReqwestClient,
}

impl Client {
    pub fn build(user_agent: Option<String>, proxy: Option<&str>) -> Result<Self, reqwest::Error> {
        let mut client_builder = ReqwestClient::builder();

        if let Some(user_agent) = user_agent {
            client_builder = client_builder.user_agent(user_agent);
        }

        if let Some(proxy) = proxy {
            client_builder = client_builder.proxy(reqwest::Proxy::all(proxy)?);
        }

        let client = client_builder.build()?;

        Ok(Client { client })
    }

    fn request(&self, method: Method, url: &str, headers: Option<HeaderMap>) -> RequestBuilder {
        let mut req = self.client.request(method, url);

        if let Some(headers) = headers {
            req = req.headers(headers);
        }

        req
    }

    pub async fn get(
        &self,
        url: &str,
        headers: Option<HeaderMap>,
    ) -> Result<Bytes, reqwest::Error> {
        let resp = self.request(Method::GET, url, headers).send().await?;

        resp.bytes().await
    }
}
