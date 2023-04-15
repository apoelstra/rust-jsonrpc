
use async_trait::async_trait;
use hyper;

use crate::{json, AsyncTransport, Client, Error};

/// Transport using a [hyper] HTTP client.
pub struct HyperTransport<C> {
    client: hyper::Client<C>,
    url: String,
    /// The value of the `Authorization` HTTP header.
    basic_auth: Option<String>,
}

impl<C> HyperTransport<C>
where
    C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    async fn request<R>(&self, req: impl serde::Serialize) -> Result<R, Error>
    where
        R: for<'a> serde::de::Deserialize<'a>,
    {
        let body = serde_json::to_string(&req).expect("JSON serializing shouldn't fail");
        let mut builder = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(&self.url);
        if let Some(ref auth) = self.basic_auth {
            builder = builder.header("Authorization", auth);
        }
        let req = builder.body(body.into())
            .map_err(|e| Error::Transport(Box::new(e)))?;
        let resp = self.client.request(req).await
            .map_err(|e| Error::Transport(Box::new(e)))?;
        let body = hyper::body::to_bytes(resp.into_body()).await
            .map_err(|e| Error::Transport(Box::new(e)))?;
        Ok(serde_json::from_reader(&body[..])?)
    }
}

#[async_trait]
impl<C> AsyncTransport for HyperTransport<C>
where
    C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    async fn send_request(
        &self,
        request: &json::Request<'_>,
    ) -> Result<json::Response, Error> {
        Ok(self.request(request).await?)
    }

    async fn send_batch(
        &self,
        requests: &[json::Request<'_>],
    ) -> Result<Vec<json::Response>, Error> {
        Ok(self.request(requests).await?)
    }
}

impl<C> Client<HyperTransport<C>> {
    /// Create a new JSON-RPC client using a bare-minimum HTTP transport.
    pub fn with_hyper(
        client: hyper::Client<C>,
        url: String,
        user: Option<String>,
        pass: Option<String>,
    ) -> Client<HyperTransport<C>> {
        let basic_auth = if let Some(user) = user {
            let mut auth = user;
            auth.push(':');
            if let Some(pass) = pass {
                auth.push_str(&pass);
            }
            Some(format!("Basic {}", &base64::encode(auth.as_bytes())))
        } else {
            None
        };

        Client::new(HyperTransport { client, url, basic_auth })
    }
}
