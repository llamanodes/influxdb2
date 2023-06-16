//! Buckets API

use reqwest::Method;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::models::{Buckets, PostBucketRequest};
use crate::{Client, HttpSnafu, RequestError, ReqwestProcessingSnafu, SerializingSnafu};

impl Client {
    /// List all buckets matching specified parameters
    pub async fn list_buckets(
        &self,
        request: Option<ListBucketsRequest>,
    ) -> Result<Buckets, RequestError> {
        let qs = serde_qs::to_string(&request).unwrap();
        let mut endpoint = "/api/v2/buckets".to_owned();
        if !qs.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&qs);
        }
        let url = self.url(&endpoint);

        let response = self
            .request(Method::GET, &url)
            .send()
            .await
            .context(ReqwestProcessingSnafu)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.context(ReqwestProcessingSnafu)?;
            let res = HttpSnafu { status, text }.fail();
            return res;
        }

        let res = response
            .json::<Buckets>()
            .await
            .context(ReqwestProcessingSnafu)?;

        Ok(res)
    }

    /// Create a new bucket in the organization specified by the 16-digit
    /// hexadecimal `org_id` and with the bucket name `bucket`.
    pub async fn create_bucket(
        &self,
        post_bucket_request: Option<PostBucketRequest>,
    ) -> Result<(), RequestError> {
        let create_bucket_url = self.url("/api/v2/buckets");

        let response = self
            .request(Method::POST, &create_bucket_url)
            .body(
                serde_json::to_string(&post_bucket_request.unwrap_or_default())
                    .context(SerializingSnafu)?,
            )
            .send()
            .await
            .context(ReqwestProcessingSnafu)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.context(ReqwestProcessingSnafu)?;
            HttpSnafu { status, text }.fail()?;
        }

        Ok(())
    }

    /// Delete a bucket specified by bucket id.
    pub async fn delete_bucket(&self, bucket_id: &str) -> Result<(), RequestError> {
        let url = self.url(&format!("/api/v2/buckets/{}", bucket_id));

        let response = self
            .request(Method::DELETE, &url)
            .send()
            .await
            .context(ReqwestProcessingSnafu)?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.context(ReqwestProcessingSnafu)?;
            HttpSnafu { status, text }.fail()?;
        }
        Ok(())
    }
}

/// Request for list buckets API
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsRequest {
    /// The last bucket ID from which to seek from (but not including). This
    /// is to be used instead of `offset`.
    pub after: Option<String>,
    /// Only returns buckets with a specific ID.
    pub id: Option<String>,
    /// Number of buckets to return. Default: 20. Valid values: [1..100]
    pub limit: Option<u8>,
    /// Only returns buckets with a specific name.
    pub name: Option<String>,
    /// Offset from which to return buckets.
    pub offset: Option<u64>,
    /// The name of the organization.
    pub org: Option<String>,
    #[serde(rename = "orgID")]
    /// The organization ID.
    pub org_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_bucket() {
        let org_id = "0000111100001111".to_string();
        let bucket = "some-bucket".to_string();
        let token = "some-token";

        let mut server = mockito::Server::new();

        let mock_server = server
            .mock("POST", "/api/v2/buckets")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_body(
                format!(
                    r#"{{"orgID":"{}","name":"{}","retentionRules":[]}}"#,
                    org_id, bucket
                )
                .as_str(),
            )
            .create();

        let client = Client::new(server.url(), &org_id, token);

        let _result = client
            .create_bucket(Some(PostBucketRequest::new(org_id, bucket)))
            .await;

        mock_server.assert();
    }

    #[test]
    fn serialize_empty_list_buckets_request() {
        let request: Option<ListBucketsRequest> = None;
        assert_eq!(serde_qs::to_string(&request).unwrap(), "");
    }
}
