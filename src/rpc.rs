use super::*;
use anyhow::Ok;
use http::header;
use reqwest::Client;

pub struct BRCZeroRpcClient {
    pub client: Client,
    pub url: String,
}

impl BRCZeroRpcClient {
    pub fn new(url: &String) -> Result<BRCZeroRpcClient> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );
        let builder = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        let client = BRCZeroRpcClient {
            client: builder,
            url: url.clone(),
        };
        Ok(client)
    }
}
