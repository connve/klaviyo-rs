//! Errors returned by the Klaviyo SDK.

use std::collections::BTreeMap;
use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// All errors surfaced by the SDK.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// No API key was provided when building the client.
    #[error("API key is required to call Klaviyo")]
    MissingApiKey,

    /// The API key contains characters that can't be sent in an HTTP header.
    #[error("API key contains characters that are not allowed in an HTTP header")]
    InvalidApiKey,

    /// The base URL could not be parsed, or can't carry a path.
    #[error("invalid Klaviyo base URL")]
    InvalidBaseUrl {
        /// Underlying URL parse error.
        #[source]
        source: url::ParseError,
    },

    /// A transport-layer failure (connection reset, TLS, DNS, timeout, etc.).
    #[error("HTTP transport error")]
    Http {
        /// Underlying reqwest error.
        #[source]
        source: reqwest::Error,
    },

    /// The Klaviyo API returned a non-success status. The message leaves out
    /// `body` and error details, which can contain personal data.
    #[error("Klaviyo API returned {status}{}", error_codes(.errors))]
    Api {
        /// HTTP status code from the response.
        status: StatusCode,
        /// The JSON:API `errors` array, empty if the body didn't carry one.
        errors: Vec<ApiError>,
        /// The `Retry-After` delay Klaviyo sent with a 429 or 503, if any.
        retry_after: Option<Duration>,
        /// Raw response body.
        body: String,
    },

    /// The request body could not be serialized to JSON.
    #[error("failed to serialize request body")]
    Serialize {
        /// Underlying serde error.
        #[source]
        source: serde_json::Error,
    },

    /// The response body could not be deserialized into the expected type.
    #[error("failed to deserialize Klaviyo response")]
    Deserialize {
        /// Underlying serde error.
        #[source]
        source: serde_json::Error,
    },

    /// A page's `links.next` has no usable cursor, or repeats the current one.
    #[error("cannot follow the next-page link `{link}`")]
    InvalidNextPage {
        /// The `links.next` value, or the repeated cursor.
        link: String,
    },

    /// A request header is invalid, or reserved by the SDK (see
    /// [`Builder::default_headers`](crate::Builder::default_headers)).
    #[error("invalid request header `{name}`")]
    InvalidHeader {
        /// The header name that was rejected.
        name: String,
    },

    /// An ID used in the request path is empty, `.`, `..`, or contains
    /// control characters.
    #[error("invalid path segment `{segment}`")]
    InvalidPathSegment {
        /// The rejected value.
        segment: String,
    },
}

/// One entry of the JSON:API `errors` array Klaviyo returns on failure.
#[derive(PartialEq, Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApiError {
    /// Unique ID of this occurrence, useful when contacting Klaviyo support.
    #[serde(default)]
    pub id: String,
    /// Machine-readable error code, e.g. `invalid`, `not_found`, `throttled`.
    #[serde(default)]
    pub code: String,
    /// Short summary of the problem.
    #[serde(default)]
    pub title: String,
    /// Explanation of this occurrence. Can echo request data.
    #[serde(default)]
    pub detail: String,
    /// Which part of the request caused the error.
    #[serde(default)]
    pub source: Option<ApiErrorSource>,
    /// Fields this SDK doesn't model, such as `status` and `meta`.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Location of the offending part of a request.
#[derive(PartialEq, Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApiErrorSource {
    /// JSON pointer into the request body, e.g. `/data/attributes/name`.
    #[serde(default)]
    pub pointer: Option<String>,
    /// Name of the offending query parameter.
    #[serde(default)]
    pub parameter: Option<String>,
    /// Fields this SDK doesn't model.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// `": invalid (Invalid input.), ..."`, or nothing.
fn error_codes(errors: &[ApiError]) -> String {
    if errors.is_empty() {
        return String::new();
    }
    let codes: Vec<String> = errors
        .iter()
        .map(|error| format!("{} ({})", error.code, error.title))
        .collect();
    format!(": {}", codes.join(", "))
}

impl Error {
    /// `true` for transport failures, HTTP 408 / 425 / 429, and 5xx.
    ///
    /// Klaviyo may have processed the request before a timeout or 5xx.
    /// Creates, campaign sends, and tracked events aren't idempotent, so
    /// retrying them can duplicate or send twice.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Http { source } => {
                source.is_timeout() || source.is_connect() || source.is_request()
            }
            Error::Api { status, .. } => {
                status.is_server_error()
                    || matches!(
                        *status,
                        StatusCode::REQUEST_TIMEOUT
                            | StatusCode::TOO_EARLY
                            | StatusCode::TOO_MANY_REQUESTS
                    )
            }
            Error::MissingApiKey
            | Error::InvalidApiKey
            | Error::InvalidBaseUrl { .. }
            | Error::Serialize { .. }
            | Error::Deserialize { .. }
            | Error::InvalidNextPage { .. }
            | Error::InvalidHeader { .. }
            | Error::InvalidPathSegment { .. } => false,
        }
    }

    /// The `Retry-After` delay of a throttled (429) request. Prefer it over
    /// your own backoff.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Error::Api { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

/// Shorthand `Result` returned by SDK methods.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    fn api_error(status: StatusCode) -> Error {
        Error::Api {
            status,
            errors: Vec::new(),
            retry_after: None,
            body: String::new(),
        }
    }

    #[test]
    fn retryable_classification() {
        assert!(api_error(StatusCode::TOO_MANY_REQUESTS).is_retryable());
        assert!(api_error(StatusCode::BAD_GATEWAY).is_retryable());
        assert!(!api_error(StatusCode::UNAUTHORIZED).is_retryable());
        assert!(!api_error(StatusCode::CONFLICT).is_retryable());
        assert!(!Error::MissingApiKey.is_retryable());
        assert!(!Error::InvalidNextPage {
            link: String::new()
        }
        .is_retryable());
    }

    #[test]
    fn retry_after_is_only_reported_for_api_errors() {
        let throttled = Error::Api {
            status: StatusCode::TOO_MANY_REQUESTS,
            errors: Vec::new(),
            retry_after: Some(Duration::from_secs(7)),
            body: String::new(),
        };
        assert_eq!(throttled.retry_after(), Some(Duration::from_secs(7)));
        assert_eq!(Error::MissingApiKey.retry_after(), None);
    }

    #[test]
    fn api_error_keeps_fields_it_does_not_model() {
        let error: ApiError = serde_json::from_str(
            r#"{ "id": "e1", "status": 400, "code": "invalid", "title": "Invalid input.",
                 "detail": "x", "meta": { "hint": "y" } }"#,
        )
        .expect("error object parses");
        assert_eq!(error.extra.get("status"), Some(&Value::from(400)));
        assert!(error.extra.contains_key("meta"));
    }

    #[test]
    fn api_error_message_leaves_out_body_and_detail() {
        let error = Error::Api {
            status: StatusCode::BAD_REQUEST,
            errors: vec![ApiError {
                code: "invalid".into(),
                title: "Invalid input.".into(),
                detail: "ada@example.com is not a valid phone number".into(),
                ..Default::default()
            }],
            retry_after: None,
            body: "ada@example.com".into(),
        };
        let message = error.to_string();
        assert!(!message.contains("ada@example.com"), "{message}");
    }
}
