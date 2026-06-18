use crate::error::{RequestRejectedError, TransportError, UnexpectedResponseError};
use reqwest::{Client, Method, Response, StatusCode};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Internal HTTP client for a single Polymarket service.
#[derive(Clone, Debug)]
pub struct ServiceClient {
    inner: Client,
    base_url: String,
}

impl ServiceClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, TransportError> {
        let inner = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .user_agent(USER_AGENT)
            .https_only(true)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| TransportError(e.to_string()))?;

        Ok(Self {
            inner,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        })
    }

    pub async fn get(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Response, TransportError> {
        self.request(Method::GET, path, query).await
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Response, TransportError> {
        let url = self.url(path);
        self.inner
            .request(method, url)
            .query(query)
            .send()
            .await
            .map_err(TransportError::from)
    }

    fn url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.base_url, path)
    }

    pub async fn ensure_success(response: Response) -> Result<Response, RequestRejectedError> {
        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(RequestRejectedError {
                status: status.as_u16(),
                message: "rate limit exceeded".into(),
            });
        }
        if status.is_success() {
            return Ok(response);
        }

        let status_code = status.as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to read error body".into());

        Err(RequestRejectedError {
            status: status_code,
            message: sanitize_error_body(&message),
        })
    }

    pub async fn json<T: serde::de::DeserializeOwned>(
        response: Response,
    ) -> Result<T, UnexpectedResponseError> {
        let text = response
            .text()
            .await
            .map_err(|e| UnexpectedResponseError(e.to_string()))?;
        serde_json::from_str(&text)
            .map_err(|e| UnexpectedResponseError(format!("failed to parse JSON response: {e}")))
    }
}

/// Avoid leaking HTML error pages into logs.
fn sanitize_error_body(body: &str) -> String {
    const MAX_LEN: usize = 512;
    if body.contains("<!DOCTYPE html") || body.contains("<html") {
        return "upstream service returned an HTML error page".into();
    }
    if body.len() > MAX_LEN {
        format!("{}…", &body[..MAX_LEN])
    } else {
        body.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_html_errors() {
        let msg = sanitize_error_body("<!DOCTYPE html><html>error</html>");
        assert!(!msg.contains("<html"));
    }
}
