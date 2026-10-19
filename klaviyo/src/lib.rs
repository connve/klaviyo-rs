//! # klaviyo
//!
//! Unofficial Rust SDK for Klaviyo APIs.
//!
//! ```no_run
//! use klaviyo::Client;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .api_key(std::env::var("KLAVIYO_API_KEY")?)
//!     .build()?;
//! # let _ = client;
//! # Ok(())
//! # }
//! ```
//!
//! Per-API examples are in the module docs.
//!
//! A private API key belongs to one account; use one [`Client`] per account.
//! Every request sends the `revision` header [`API_REVISION`], the revision
//! [`types`] were generated from.
//!
//! Endpoint methods return a [`RequestBuilder`] typed by the operation's
//! marker in [`operations`], so it offers only the query options the
//! operation accepts. The SDK doesn't retry; see [`Error::is_retryable`] and
//! [`Error::retry_after`].
//!
//! ## Cargo features
//!
//! None are enabled by default.
//!
//! - `accounts`, `campaigns`, `events`, `flows`, `lists`, `metrics`,
//!   `profiles`, `reporting`, `segments`, `templates` — one per API.
//! - `trace` — `#[tracing::instrument]` on public async methods.
//!
//! ```toml
//! klaviyo = { version = "0.2", features = ["campaigns", "templates"] }
//! ```

#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Re-export, so callers use the same `chrono` version as the generated types.
pub use chrono;

/// Re-export, so callers use the same `reqwest` version as
/// [`Builder::default_headers`] and `.header()` / `.headers()`.
pub use reqwest::header;

/// Re-export; holds [`Credentials::api_key`].
pub use secrecy;

/// Re-export; `Value` appears in the generated types for free-form fields.
pub use serde_json;

/// The API revision sent with every request.
pub use klaviyo_generated::API_REVISION;

/// Per-operation markers, generated from the spec; the last type parameter
/// of every [`RequestBuilder`].
pub use klaviyo_generated::operations;

/// Request and response types generated from the spec.
///
/// Request types implement `Default` where their required fields allow it,
/// with JSON:API `type_` fields defaulting to the right resource type, so
/// literals can end with `..Default::default()`.
pub use klaviyo_generated::types;

mod client;
mod error;
mod headers;
#[cfg(feature = "_api")]
mod http;
#[cfg(feature = "_api")]
mod request;

#[cfg(feature = "accounts")]
pub mod accounts;
#[cfg(feature = "campaigns")]
pub mod campaigns;
#[cfg(feature = "events")]
pub mod events;
#[cfg(feature = "flows")]
pub mod flows;
#[cfg(feature = "lists")]
pub mod lists;
#[cfg(feature = "metrics")]
pub mod metrics;
#[cfg(feature = "profiles")]
pub mod profiles;
#[cfg(feature = "reporting")]
pub mod reporting;
#[cfg(feature = "segments")]
pub mod segments;
#[cfg(feature = "templates")]
pub mod templates;

pub use client::{Builder, Client, Credentials, DEFAULT_BASE_URL};
pub use error::{ApiError, ApiErrorSource, Error, Result};
#[cfg(feature = "_api")]
pub use request::{NoContent, Pages, RequestBuilder};

/// The raw `progenitor` crate, including its client.
pub use klaviyo_generated as generated;
