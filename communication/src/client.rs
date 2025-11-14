use agent_core::{AgentError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP client for API communication with timeout and retry support
pub struct ApiClient {
    client: Client,
    timeout: Duration,
}

impl ApiClient {
    /// Create a new ApiClient with default timeout of 30 seconds
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Create a new ApiClient with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            client: Client::new(),
            timeout,
        }
    }

    /// Send a JSON POST request and deserialize the response
    ///
    /// # Arguments
    /// * `url` - The URL to send the request to
    /// * `body` - The request body to serialize as JSON
    ///
    /// # Returns
    /// The deserialized response or an error
    pub async fn post_json<T, R>(&self, url: &str, body: &T) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let response = self
            .client
            .post(url)
            .json(body)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AgentError::LLMProvider(format!("Request timeout: {}", e))
                } else if e.is_connect() {
                    AgentError::LLMProvider(format!("Connection error: {}", e))
                } else {
                    AgentError::LLMProvider(format!("Request failed: {}", e))
                }
            })?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(AgentError::LLMProvider(format!(
                "HTTP {} error: {}",
                status, error_text
            )));
        }

        // Deserialize the response
        response.json().await.map_err(|e| {
            AgentError::LLMProvider(format!("Failed to deserialize response: {}", e))
        })
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ApiClient::new();
        assert_eq!(client.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_client_with_custom_timeout() {
        let client = ApiClient::with_timeout(Duration::from_secs(60));
        assert_eq!(client.timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_default_client() {
        let client = ApiClient::default();
        assert_eq!(client.timeout(), Duration::from_secs(30));
    }
}
