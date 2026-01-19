// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! HTTP client wrapper with authentication and `ETag` handling.

use reqwest::{Client, RequestBuilder, Response};

use crate::config::{AuthMethod, CalDavConfig};
use crate::error::CalDavError;
use crate::types::ETag;

/// HTTP client for `CalDAV` operations.
#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    config: CalDavConfig,
}

impl HttpClient {
    /// Creates a new HTTP client.
    ///
    /// # Errors
    ///
    /// Returns an error if HTTP client creation fails.
    pub fn new(config: CalDavConfig) -> Result<Self, CalDavError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .build()?;
        Ok(Self { client, config })
    }

    /// Builds a request with authentication headers.
    pub fn build_request(&self, method: reqwest::Method, url: &str) -> RequestBuilder {
        let mut req = self.client.request(method, url);

        match &self.config.auth {
            AuthMethod::Basic { username, password } => {
                req = req.basic_auth(username, Some(password));
            }
            AuthMethod::Bearer { token } => {
                req = req.bearer_auth(token);
            }
            AuthMethod::None => {}
        }

        req
    }

    /// Executes a request and checks for HTTP errors.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or returns an error status code.
    pub async fn execute(&self, req: RequestBuilder) -> Result<Response, CalDavError> {
        let resp = req.send().await?;

        match resp.status() {
            reqwest::StatusCode::OK
            | reqwest::StatusCode::CREATED
            | reqwest::StatusCode::NO_CONTENT
            | reqwest::StatusCode::MULTI_STATUS => Ok(resp),
            reqwest::StatusCode::PRECONDITION_FAILED => Err(CalDavError::PreconditionFailed(
                resp.headers()
                    .get("ETag")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string(),
            )),
            status => {
                let text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unable to read response".to_string());
                Err(CalDavError::Http(format!("{status}: {text}")))
            }
        }
    }

    /// Adds If-Match header for conditional updates.
    pub fn if_match(req: RequestBuilder, etag: &ETag) -> RequestBuilder {
        req.header("If-Match", etag.as_str())
    }

    /// Adds If-None-Match header for conditional creation.
    #[expect(dead_code)]
    pub fn if_none_match(req: RequestBuilder, etag: &ETag) -> RequestBuilder {
        req.header("If-None-Match", etag.as_str())
    }

    /// Extracts `ETag` from response headers.
    ///
    /// # Errors
    ///
    /// Returns an error if the `ETag` header is missing.
    pub fn extract_etag(resp: &Response) -> Result<ETag, CalDavError> {
        resp.headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|s| ETag::new(s.to_string()))
            .ok_or_else(|| CalDavError::Http("Missing ETag header".to_string()))
    }
}
