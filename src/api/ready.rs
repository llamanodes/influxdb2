//! Ready
//!
//! Check readiness of an InfluxDB instance at startup

use reqwest::{Method, StatusCode};
use snafu::ResultExt;

use crate::{Client, HttpSnafu, RequestError, ReqwestProcessingSnafu};

impl Client {
    /// Get the readiness of an instance at startup
    pub async fn ready(&self) -> Result<bool, RequestError> {
        let ready_url = self.url("/ready");
        let response = self
            .request(Method::GET, &ready_url)
            .send()
            .await
            .context(ReqwestProcessingSnafu)?;

        match response.status() {
            StatusCode::OK => Ok(true),
            _ => {
                let status = response.status();
                let text = response.text().await.context(ReqwestProcessingSnafu)?;
                HttpSnafu { status, text }.fail()?
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ready() {
        let mut server = mockito::Server::new();

        let mock_server = server.mock("GET", "/ready").create();

        let client = Client::new(server.url(), "org", "");

        let _result = client.ready().await;

        mock_server.assert();
    }
}
