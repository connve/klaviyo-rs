//! Helpers for live tests against the test account whose private API key is
//! in `KLAVIYO_API_KEY`.
//!
//! The checks return an error instead of panicking, so cleanup still runs.
//! They are macros so files that use only some of them don't hit
//! `dead_code`.

use klaviyo::Client;
use std::env;

/// The test account's private API key.
const API_KEY_ENV: &str = "KLAVIYO_API_KEY";

/// A client for the test account. Fails when `KLAVIYO_API_KEY` is unset, so
/// an ignored test run without it can't pass.
pub fn client() -> Result<Client, Box<dyn std::error::Error>> {
    let api_key = env::var(API_KEY_ENV).map_err(|error| format!("{API_KEY_ENV}: {error}"))?;
    let client = Client::builder().api_key(api_key).build()?;
    Ok(client)
}

/// `assert!` that returns an error instead of panicking.
#[macro_export]
macro_rules! ensure {
    ($condition:expr, $($message:tt)+) => {
        if !$condition {
            return Err(format!($($message)+).into());
        }
    };
}

/// Expects a non-retryable 4xx other than 401/403: the key worked and
/// Klaviyo rejected the request itself.
#[macro_export]
macro_rules! ensure_rejected {
    ($result:expr) => {
        match $result {
            Err(error) => {
                let rejected = matches!(
                    &error,
                    klaviyo::Error::Api { status, .. }
                        if status.is_client_error()
                            && status.as_u16() != 401
                            && status.as_u16() != 403
                );
                if !rejected || error.is_retryable() {
                    return Err(format!(
                        "expected a non-retryable 4xx other than 401/403, got: {error:?}"
                    )
                    .into());
                }
            }
            Ok(value) => {
                return Err(format!("expected a rejection, got: {value:?}").into());
            }
        }
    };
}

/// A name no other run uses, for unknown IDs and created resources.
#[macro_export]
macro_rules! unique_name {
    ($prefix:expr) => {
        format!(
            "klaviyo-rs-test-{}-{}",
            $prefix,
            ::chrono::Utc::now().format("%Y%m%d%H%M%S%9f")
        )
    };
}
