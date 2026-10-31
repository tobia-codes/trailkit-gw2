//! Polite blocking client for the public Guild Wars 2 API (data-pipeline D26,
//! ROADMAP R65).
//!
//! Every request is throttled to [`MIN_REQUEST_INTERVAL`] and retried with
//! exponential backoff on rate limiting (429), server errors (5xx) and
//! transport errors (D35).

use std::{
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};

/// Base URL of the official GW2 API.
const BASE_URL: &str = "https://api.guildwars2.com";

/// User agent identifying the tool and where to find its source.
const USER_AGENT: &str = concat!(
    "trailkit-datagen/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/tobia-codes/trailkit-gw2)"
);

/// Minimum time between two requests (~4 requests per second, D26).
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(250);

/// Attempts per request before giving up (D35).
const MAX_ATTEMPTS: u32 = 5;

/// Wait time before the first retry; doubles on every further retry.
const FIRST_BACKOFF: Duration = Duration::from_secs(1);

/// Upper bound for the wait time between two retries.
const MAX_BACKOFF: Duration = Duration::from_secs(60);

/// Timeout for one complete request including the body.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Largest accepted response body (order book batches are a few MB).
const MAX_BODY_BYTES: u64 = 256 * 1024 * 1024;

/// Maximum number of ids the API accepts in one `?ids=` request.
pub const MAX_IDS_PER_REQUEST: usize = 200;

/// Result of a successful request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiResponse {
    /// 2xx (including 206 "some ids were invalid") with its body.
    Found(String),
    /// 404, e.g. none of the requested ids exists (anymore).
    NotFound,
}

/// Why one attempt failed.
#[derive(Debug)]
enum AttemptError {
    /// Worth retrying (429, 5xx, transport error).
    Retryable(String),
    /// Retrying cannot help (other 4xx).
    Fatal(String),
}

/// Throttled, retrying GW2 API client.
pub struct ApiClient {
    /// Underlying HTTP agent (connection pool, timeouts, user agent).
    agent: ureq::Agent,
    /// When the previous request was started, for the throttle.
    last_request: Option<Instant>,
}

impl ApiClient {
    /// Creates a client for the official API.
    #[must_use]
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(REQUEST_TIMEOUT))
            .user_agent(USER_AGENT)
            .build();
        Self {
            agent: ureq::Agent::new_with_config(config),
            last_request: None,
        }
    }

    /// Sends `GET <base><path>` (path including the query) with throttle and
    /// retries.
    ///
    /// # Errors
    /// Returns an error on a non-retryable status or when every attempt failed.
    pub fn get(&mut self, path: &str) -> Result<ApiResponse> {
        let url = format!("{BASE_URL}{path}");
        let mut backoff = FIRST_BACKOFF;
        let mut attempt = 1;
        loop {
            match self.attempt(&url) {
                Ok(response) => return Ok(response),
                Err(AttemptError::Fatal(message)) => bail!("GET {url}: {message}"),
                Err(AttemptError::Retryable(message)) if attempt >= MAX_ATTEMPTS => {
                    bail!("GET {url}: {message} (after {MAX_ATTEMPTS} attempts)")
                }
                Err(AttemptError::Retryable(message)) => {
                    log::warn!("GET {url}: {message}; retry {attempt} in {backoff:?}");
                    thread::sleep(backoff);
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                    attempt += 1;
                }
            }
        }
    }

    /// Fetches an id list endpoint such as `/v2/commerce/prices`.
    ///
    /// # Errors
    /// Returns an error if the request fails, the endpoint answers 404 or the
    /// body is not a JSON array of ids.
    pub fn get_ids(&mut self, path: &str) -> Result<Vec<u32>> {
        match self.get(path)? {
            ApiResponse::Found(body) => {
                serde_json::from_str(&body).with_context(|| format!("parsing id list of {path}"))
            }
            ApiResponse::NotFound => bail!("GET {path}: 404"),
        }
    }

    /// Performs one throttled attempt.
    fn attempt(&mut self, url: &str) -> Result<ApiResponse, AttemptError> {
        self.wait_for_throttle();
        let mut response = self
            .agent
            .get(url)
            .call()
            .map_err(|error| AttemptError::Retryable(error.to_string()))?;
        let status = response.status().as_u16();
        match status {
            200..=299 => response
                .body_mut()
                .with_config()
                .limit(MAX_BODY_BYTES)
                .read_to_string()
                .map(ApiResponse::Found)
                .map_err(|error| AttemptError::Retryable(format!("reading body: {error}"))),
            404 => Ok(ApiResponse::NotFound),
            429 | 500..=599 => Err(AttemptError::Retryable(format!("HTTP {status}"))),
            _ => Err(AttemptError::Fatal(format!("HTTP {status}"))),
        }
    }

    /// Sleeps until [`MIN_REQUEST_INTERVAL`] has passed since the last request.
    fn wait_for_throttle(&mut self) {
        if let Some(last_request) = self.last_request {
            let elapsed = last_request.elapsed();
            if elapsed < MIN_REQUEST_INTERVAL {
                thread::sleep(MIN_REQUEST_INTERVAL - elapsed);
            }
        }
        self.last_request = Some(Instant::now());
    }
}

/// Builds the query path for one batch of ids, e.g. `/v2/commerce/prices?ids=1,2`.
#[must_use]
pub fn ids_path(endpoint: &str, ids: &[u32]) -> String {
    let joined: Vec<String> = ids.iter().map(u32::to_string).collect();
    format!("{endpoint}?ids={}", joined.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_path_joins_ids() {
        assert_eq!(
            ids_path("/v2/commerce/prices", &[24, 19721]),
            "/v2/commerce/prices?ids=24,19721"
        );
    }
}
