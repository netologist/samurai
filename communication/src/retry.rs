use agent_core::{AgentError, Result};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Retry an async operation with exponential backoff
///
/// # Arguments
/// * `operation` - The async operation to retry
/// * `max_attempts` - Maximum number of attempts (default: 3)
///
/// # Returns
/// The result of the operation or the last error encountered
///
/// # Behavior
/// - Initial delay: 1 second
/// - Backoff strategy: Exponential (doubles each retry)
/// - Only retries on network errors and 5xx status codes
/// - Does not retry on 4xx errors (client errors)
pub async fn with_retry<F, Fut, T>(mut operation: F, max_attempts: u32) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let max_attempts = if max_attempts == 0 { 3 } else { max_attempts };
    let mut attempt = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Check if we should retry
                let should_retry = should_retry_error(&e);

                if attempt >= max_attempts || !should_retry {
                    return Err(e);
                }

                // Wait before retrying with exponential backoff
                sleep(delay).await;
                delay *= 2;
            }
        }
    }
}

/// Determine if an error should trigger a retry
///
/// Retries on:
/// - Network/connection errors
/// - Timeout errors
/// - 5xx server errors
///
/// Does not retry on:
/// - 4xx client errors (bad request, auth failure, etc.)
/// - Serialization errors
/// - Other non-transient errors
fn should_retry_error(error: &AgentError) -> bool {
    match error {
        AgentError::LLMProvider(msg) => {
            // Retry on network, connection, and timeout errors
            msg.contains("timeout")
                || msg.contains("Connection error")
                || msg.contains("HTTP 5") // 5xx errors
        }
        // Don't retry on other error types
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let result = with_retry(|| async { Ok::<i32, AgentError>(42) }, 3).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = with_retry(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(AgentError::LLMProvider("Connection error".to_string()))
                    } else {
                        Ok(42)
                    }
                }
            },
            3,
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = with_retry(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, AgentError>(AgentError::LLMProvider(
                        "Request timeout".to_string(),
                    ))
                }
            },
            3,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_no_retry_on_client_error() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = with_retry(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, AgentError>(AgentError::LLMProvider(
                        "HTTP 400 error: Bad Request".to_string(),
                    ))
                }
            },
            3,
        )
        .await;

        assert!(result.is_err());
        // Should only attempt once, no retries for 4xx errors
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_on_5xx_error() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = with_retry(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 1 {
                        Err(AgentError::LLMProvider("HTTP 503 error: Service Unavailable".to_string()))
                    } else {
                        Ok(42)
                    }
                }
            },
            3,
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_should_retry_error() {
        // Should retry
        assert!(should_retry_error(&AgentError::LLMProvider(
            "Request timeout".to_string()
        )));
        assert!(should_retry_error(&AgentError::LLMProvider(
            "Connection error".to_string()
        )));
        assert!(should_retry_error(&AgentError::LLMProvider(
            "HTTP 500 error".to_string()
        )));
        assert!(should_retry_error(&AgentError::LLMProvider(
            "HTTP 503 error".to_string()
        )));

        // Should not retry
        assert!(!should_retry_error(&AgentError::LLMProvider(
            "HTTP 400 error".to_string()
        )));
        assert!(!should_retry_error(&AgentError::LLMProvider(
            "HTTP 401 error".to_string()
        )));
        assert!(!should_retry_error(&AgentError::Config(
            "Invalid config".to_string()
        )));
        assert!(!should_retry_error(&AgentError::Planning(
            "Planning failed".to_string()
        )));
    }
}
