//! Klaviyo client, credentials, and builder.

#[cfg(feature = "_api")]
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use url::Url;

use crate::error::{Error, Result};
use crate::headers;

/// Klaviyo's API host. The API key selects the account.
pub const DEFAULT_BASE_URL: &str = "https://a.klaviyo.com";

/// Authentication for one Klaviyo account.
///
/// Deserializes from `{ "api_key": "pk_..." }`, ignoring other fields, so an
/// application can load it from wherever it keeps secrets.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    /// Private API key (`pk_...`) from *Settings → API keys*. Redacted from
    /// `Debug` output and zeroed on drop.
    pub api_key: SecretString,
}

#[cfg(feature = "_api")]
#[derive(Debug)]
struct Inner {
    authorization: HeaderValue,
    base_url: Url,
    default_headers: HeaderMap,
    http: reqwest::Client,
}

/// Klaviyo API client for one account. Cheap to clone.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Client {
    #[cfg(feature = "_api")]
    inner: Arc<Inner>,
}

impl Client {
    /// Start a [`Builder`].
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Account endpoints.
    #[cfg(feature = "accounts")]
    pub fn accounts(&self) -> crate::accounts::Accounts<'_> {
        crate::accounts::Accounts::new(self)
    }

    /// Campaign endpoints.
    #[cfg(feature = "campaigns")]
    pub fn campaigns(&self) -> crate::campaigns::Campaigns<'_> {
        crate::campaigns::Campaigns::new(self)
    }

    /// Event endpoints.
    #[cfg(feature = "events")]
    pub fn events(&self) -> crate::events::Events<'_> {
        crate::events::Events::new(self)
    }

    /// Flow endpoints.
    #[cfg(feature = "flows")]
    pub fn flows(&self) -> crate::flows::Flows<'_> {
        crate::flows::Flows::new(self)
    }

    /// List endpoints.
    #[cfg(feature = "lists")]
    pub fn lists(&self) -> crate::lists::Lists<'_> {
        crate::lists::Lists::new(self)
    }

    /// Metric endpoints.
    #[cfg(feature = "metrics")]
    pub fn metrics(&self) -> crate::metrics::Metrics<'_> {
        crate::metrics::Metrics::new(self)
    }

    /// Profile endpoints.
    #[cfg(feature = "profiles")]
    pub fn profiles(&self) -> crate::profiles::Profiles<'_> {
        crate::profiles::Profiles::new(self)
    }

    /// Reporting endpoints.
    #[cfg(feature = "reporting")]
    pub fn reporting(&self) -> crate::reporting::Reporting<'_> {
        crate::reporting::Reporting::new(self)
    }

    /// Segment endpoints.
    #[cfg(feature = "segments")]
    pub fn segments(&self) -> crate::segments::Segments<'_> {
        crate::segments::Segments::new(self)
    }

    /// Template endpoints.
    #[cfg(feature = "templates")]
    pub fn templates(&self) -> crate::templates::Templates<'_> {
        crate::templates::Templates::new(self)
    }

    #[cfg(not(feature = "_api"))]
    fn new(
        _authorization: HeaderValue,
        _base_url: Url,
        _default_headers: HeaderMap,
    ) -> Result<Self> {
        Ok(Self {})
    }
}

#[cfg(feature = "_api")]
impl Client {
    fn new(authorization: HeaderValue, base_url: Url, default_headers: HeaderMap) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(Inner {
                authorization,
                base_url,
                default_headers,
                http: crate::http::build_reqwest_client()?,
            }),
        })
    }

    pub(crate) fn authorization(&self) -> &HeaderValue {
        &self.inner.authorization
    }

    pub(crate) fn base_url(&self) -> &Url {
        &self.inner.base_url
    }

    pub(crate) fn default_headers(&self) -> &HeaderMap {
        &self.inner.default_headers
    }

    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }
}

/// Marked sensitive, so `Debug` output of it and of reqwest requests redacts
/// the key.
fn authorization_header(api_key: &str) -> Result<HeaderValue> {
    let value = SecretString::from(format!("Klaviyo-API-Key {api_key}"));
    let mut value =
        HeaderValue::from_str(value.expose_secret()).map_err(|_| Error::InvalidApiKey)?;
    value.set_sensitive(true);
    Ok(value)
}

/// Paths are appended as segments, so the URL must be able to carry one.
fn parse_base_url(base_url: &str) -> Result<Url> {
    let url = Url::parse(base_url).map_err(|source| Error::InvalidBaseUrl { source })?;
    if url.cannot_be_a_base() {
        return Err(Error::InvalidBaseUrl {
            source: url::ParseError::RelativeUrlWithCannotBeABaseBase,
        });
    }
    Ok(url)
}

/// Fluent constructor for [`Client`].
#[derive(Debug, Default)]
pub struct Builder {
    credentials: Option<Credentials>,
    base_url: Option<String>,
    default_headers: HeaderMap,
}

impl Builder {
    /// Headers sent with every request. Per-call headers of the same name
    /// win. Reserved headers (`Authorization`, `revision`, `Content-Type`,
    /// `Content-Length`, `Transfer-Encoding`, `Accept`, `Host`, `User-Agent`)
    /// make [`Builder::build`] fail with [`Error::InvalidHeader`].
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = headers;
        self
    }

    /// The account's private API key (`pk_...`).
    pub fn api_key(self, api_key: impl Into<SecretString>) -> Self {
        self.credentials(Credentials {
            api_key: api_key.into(),
        })
    }

    /// Credentials loaded by the application.
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Use `base_url` instead of [`DEFAULT_BASE_URL`], e.g. a proxy or a mock
    /// server.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Build the client. Whitespace around the API key is ignored.
    pub fn build(self) -> Result<Client> {
        headers::reject_forbidden(&self.default_headers)?;

        let credentials = self.credentials.ok_or(Error::MissingApiKey)?;
        let api_key = credentials.api_key.expose_secret().trim();
        if api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }
        let authorization = authorization_header(api_key)?;
        let base_url = parse_base_url(self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL))?;

        Client::new(authorization, base_url, self.default_headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credentials(api_key: &str) -> Credentials {
        Credentials {
            api_key: api_key.into(),
        }
    }

    #[test]
    fn builder_rejects_empty_api_key() {
        let result = Client::builder().credentials(credentials("   ")).build();
        assert!(matches!(result, Err(Error::MissingApiKey)));
    }

    #[test]
    fn builder_rejects_missing_credentials() {
        assert!(matches!(
            Client::builder().build(),
            Err(Error::MissingApiKey)
        ));
    }

    #[test]
    fn builder_rejects_invalid_base_url() {
        let result = Client::builder()
            .credentials(credentials("pk_test"))
            .base_url("not a url")
            .build();
        assert!(matches!(result, Err(Error::InvalidBaseUrl { .. })));
    }

    #[test]
    fn builder_rejects_base_url_without_path() {
        let result = Client::builder()
            .credentials(credentials("pk_test"))
            .base_url("mailto:ops@example.com")
            .build();
        assert!(matches!(result, Err(Error::InvalidBaseUrl { .. })));
    }

    #[test]
    fn builder_rejects_api_key_not_valid_in_a_header() {
        let result = Client::builder()
            .credentials(credentials("abc\ndef"))
            .build();
        assert!(matches!(result, Err(Error::InvalidApiKey)));
    }

    #[test]
    fn credentials_ignore_extra_fields() {
        let parsed: Credentials =
            serde_json::from_str(r#"{ "api_key": "pk_test", "market": "PL" }"#)
                .expect("credentials parse");
        assert_eq!(parsed.api_key.expose_secret(), "pk_test");
    }

    #[cfg(feature = "_api")]
    #[test]
    fn builder_trims_api_key_and_uses_klaviyo_scheme() {
        let client = Client::builder()
            .api_key("pk_test\n")
            .build()
            .expect("client builds");
        assert_eq!(client.authorization(), "Klaviyo-API-Key pk_test");
        assert_eq!(client.base_url().as_str(), "https://a.klaviyo.com/");
    }

    #[cfg(feature = "_api")]
    #[test]
    fn debug_output_does_not_contain_api_key() {
        let credentials = credentials("pk_super_secret");
        let builder = Client::builder().credentials(credentials.clone());
        let client = Client::builder()
            .credentials(credentials.clone())
            .build()
            .expect("client builds");

        for output in [
            format!("{credentials:?}"),
            format!("{builder:?}"),
            format!("{client:?}"),
        ] {
            assert!(!output.contains("pk_super_secret"), "leaked: {output}");
        }
    }

    #[test]
    fn builder_rejects_reserved_default_header() {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(headers::REVISION, HeaderValue::from_static("2023-01-01"));
        let result = Client::builder()
            .credentials(credentials("pk_test"))
            .default_headers(default_headers)
            .build();
        assert!(matches!(
            result,
            Err(Error::InvalidHeader { ref name }) if name == "revision"
        ));
    }
}
