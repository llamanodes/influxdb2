//! Health
//!
//! Get health of an InfluxDB instance

use crate::models::HealthCheck;
use crate::{Client, HttpSnafu, RequestError, ReqwestProcessingSnafu};
use reqwest::{Method, StatusCode};
use snafu::ResultExt;

impl Client {
    /// Get health of an instance
    pub async fn health(&self) -> Result<HealthCheck, RequestError> {
        let health_url = self.url("/health");
        let response = self
            .request(Method::GET, &health_url)
            .send()
            .await
            .context(ReqwestProcessingSnafu)?;

        match response.status() {
            StatusCode::OK => Ok(response
                .json::<HealthCheck>()
                .await
                .context(ReqwestProcessingSnafu)?),
            StatusCode::SERVICE_UNAVAILABLE => Ok(response
                .json::<HealthCheck>()
                .await
                .context(ReqwestProcessingSnafu)?),
            status => {
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
    async fn health() {
        let mut server = mockito::Server::new();

        let mock_server = server.mock("GET", "/health").create();

        let client = Client::new(server.url(), "", "");

        let _result = client.health().await;

        mock_server.assert();
    }
}
