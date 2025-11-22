use agent_core::{AgentError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP client for API communication with timeout and retry support
#[derive(Clone)]
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
        self.post_json_with_headers(url, body, reqwest::header::HeaderMap::new())
            .await
    }

    /// Send a JSON POST request with custom headers and deserialize the response
    ///
    /// # Arguments
    /// * `url` - The URL to send the request to
    /// * `body` - The request body to serialize as JSON
    /// * `headers` - Custom headers to include in the request
    ///
    /// # Returns
    /// The deserialized response or an error
    pub async fn post_json_with_headers<T, R>(
        &self,
        url: &str,
        body: &T,
        headers: reqwest::header::HeaderMap,
    ) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let response = self
            .client
            .post(url)
            .headers(headers)
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
        response
            .json()
            .await
            .map_err(|e| AgentError::LLMProvider(format!("Failed to deserialize response: {}", e)))
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
    use serde::{Deserialize, Serialize};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestRequest {
        message: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestResponse {
        reply: String,
    }

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

    #[tokio::test]
    async fn test_successful_post_request() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up mock response
        let response_body = TestResponse {
            reply: "Hello back!".to_string(),
        };

        Mock::given(method("POST"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        // Make request
        let client = ApiClient::new();
        let request = TestRequest {
            message: "Hello".to_string(),
        };

        let url = format!("{}/test", mock_server.uri());
        let result: Result<TestResponse> = client.post_json(&url, &request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.reply, "Hello back!");
    }

    #[tokio::test]
    async fn test_http_error_handling() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up mock to return 500 error
        Mock::given(method("POST"))
            .and(path("/error"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        // Make request
        let client = ApiClient::new();
        let request = TestRequest {
            message: "Hello".to_string(),
        };

        let url = format!("{}/error", mock_server.uri());
        let result: Result<TestResponse> = client.post_json(&url, &request).await;

        assert!(result.is_err());
        match result {
            Err(AgentError::LLMProvider(msg)) => {
                assert!(msg.contains("HTTP 500"));
                assert!(msg.contains("Internal Server Error"));
            }
            _ => panic!("Expected LLMProvider error"),
        }
    }

    #[tokio::test]
    async fn test_http_404_error() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up mock to return 404 error
        Mock::given(method("POST"))
            .and(path("/notfound"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&mock_server)
            .await;

        // Make request
        let client = ApiClient::new();
        let request = TestRequest {
            message: "Hello".to_string(),
        };

        let url = format!("{}/notfound", mock_server.uri());
        let result: Result<TestResponse> = client.post_json(&url, &request).await;

        assert!(result.is_err());
        match result {
            Err(AgentError::LLMProvider(msg)) => {
                assert!(msg.contains("HTTP 404"));
            }
            _ => panic!("Expected LLMProvider error"),
        }
    }

    #[tokio::test]
    async fn test_deserialization_error() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up mock to return invalid JSON
        Mock::given(method("POST"))
            .and(path("/invalid"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
            .mount(&mock_server)
            .await;

        // Make request
        let client = ApiClient::new();
        let request = TestRequest {
            message: "Hello".to_string(),
        };

        let url = format!("{}/invalid", mock_server.uri());
        let result: Result<TestResponse> = client.post_json(&url, &request).await;

        assert!(result.is_err());
        match result {
            Err(AgentError::LLMProvider(msg)) => {
                assert!(msg.contains("Failed to deserialize response"));
            }
            _ => panic!("Expected LLMProvider error"),
        }
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up mock with long delay (longer than our timeout)
        Mock::given(method("POST"))
            .and(path("/slow"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(&TestResponse {
                        reply: "Slow response".to_string(),
                    })
                    .set_delay(Duration::from_secs(2)),
            )
            .mount(&mock_server)
            .await;

        // Create client with very short timeout
        let client = ApiClient::with_timeout(Duration::from_millis(100));
        let request = TestRequest {
            message: "Hello".to_string(),
        };

        let url = format!("{}/slow", mock_server.uri());
        let result: Result<TestResponse> = client.post_json(&url, &request).await;

        assert!(result.is_err());
        match result {
            Err(AgentError::LLMProvider(msg)) => {
                assert!(msg.contains("timeout") || msg.contains("Request timeout"));
            }
            _ => panic!("Expected LLMProvider error with timeout"),
        }
    }

    #[tokio::test]
    async fn test_connection_error() {
        // Use an invalid URL that will cause a connection error
        let client = ApiClient::new();
        let request = TestRequest {
            message: "Hello".to_string(),
        };

        // Use a URL that will fail to connect
        let url = "http://localhost:1"; // Port 1 is typically not accessible
        let result: Result<TestResponse> = client.post_json(url, &request).await;

        assert!(result.is_err());
        match result {
            Err(AgentError::LLMProvider(msg)) => {
                // Should contain either "Connection error" or "Request failed"
                assert!(
                    msg.contains("Connection error") || msg.contains("Request failed"),
                    "Unexpected error message: {}",
                    msg
                );
            }
            _ => panic!("Expected LLMProvider error"),
        }
    }

    #[tokio::test]
    async fn test_retry_with_successful_request() {
        use crate::with_retry;

        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up mock response
        let response_body = TestResponse {
            reply: "Success".to_string(),
        };

        Mock::given(method("POST"))
            .and(path("/retry-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        // Make request with retry
        let client = ApiClient::new();

        let url = format!("{}/retry-test", mock_server.uri());
        let result: Result<TestResponse> = with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                let request = TestRequest {
                    message: "Hello".to_string(),
                };
                async move { client.post_json(&url, &request).await }
            },
            3,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.reply, "Success");
    }

    #[tokio::test]
    async fn test_retry_with_eventual_success() {
        use crate::with_retry;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};

        // Start a mock server
        let mock_server = MockServer::start().await;

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        // Set up mock that fails twice then succeeds
        Mock::given(method("POST"))
            .and(path("/retry-eventual"))
            .respond_with(move |_req: &wiremock::Request| {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    ResponseTemplate::new(503).set_body_string("Service Unavailable")
                } else {
                    ResponseTemplate::new(200).set_body_json(&TestResponse {
                        reply: "Finally!".to_string(),
                    })
                }
            })
            .mount(&mock_server)
            .await;

        // Make request with retry
        let client = ApiClient::new();
        let url = format!("{}/retry-eventual", mock_server.uri());

        let result: Result<TestResponse> = with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                let request = TestRequest {
                    message: "Hello".to_string(),
                };
                async move { client.post_json(&url, &request).await }
            },
            3,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.reply, "Finally!");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
