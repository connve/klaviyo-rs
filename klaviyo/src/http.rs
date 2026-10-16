//! Shared HTTP plumbing used by every API module.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, RETRY_AFTER};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use url::Url;

use crate::client::Client;
use crate::error::{ApiError, Error, Result};
use crate::headers::REVISION;

/// Sent with every request so Klaviyo can tell this SDK's traffic apart.
pub(crate) const USER_AGENT: &str = concat!("klaviyo-rs/", env!("CARGO_PKG_VERSION"));

/// The media type of every Klaviyo request and response body.
const JSON_API: &str = "application/vnd.api+json";

/// Total time allowed for one request, including reading the response.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Time allowed to establish the connection.
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Caller default headers are merged per request in [`send`], not set here:
/// reqwest keeps only the last value of a multi-valued default header.
pub(crate) fn build_reqwest_client() -> Result<reqwest::Client> {
    let mut default_headers = HeaderMap::new();
    default_headers.insert(ACCEPT, HeaderValue::from_static(JSON_API));
    default_headers.insert(REVISION, HeaderValue::from_static(crate::API_REVISION));

    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(DEFAULT_TIMEOUT)
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
        .default_headers(default_headers)
        .build()
        .map_err(|source| Error::Http { source })
}

/// Each segment is percent-encoded, so an ID can't escape it. `url` would
/// silently drop `.` / `..` and strip control characters, so those are
/// rejected instead.
pub(crate) fn endpoint_url(
    base: &Url,
    segments: &[String],
    query: &[(String, String)],
) -> Result<Url> {
    if let Some(segment) = segments.iter().find(|segment| {
        segment.is_empty()
            || *segment == "."
            || *segment == ".."
            || segment.contains(char::is_control)
    }) {
        return Err(Error::InvalidPathSegment {
            segment: segment.clone(),
        });
    }

    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|()| Error::InvalidBaseUrl {
            source: url::ParseError::RelativeUrlWithCannotBeABaseBase,
        })?
        .pop_if_empty()
        .extend(segments);
    if !query.is_empty() {
        url.query_pairs_mut().extend_pairs(query);
    }
    Ok(url)
}

/// With `id`, sets `data.id`, so the path and body can't disagree.
#[cfg(feature = "_write")]
pub(crate) fn json_body<T: serde::Serialize + ?Sized>(
    body: &T,
    id: Option<&str>,
) -> Result<Vec<u8>> {
    use serde_json::Value;

    let Some(id) = id else {
        return serde_json::to_vec(body).map_err(|source| Error::Serialize { source });
    };
    let mut document = serde_json::to_value(body).map_err(|source| Error::Serialize { source })?;
    if let Some(data) = document.get_mut("data").and_then(Value::as_object_mut) {
        data.insert("id".to_owned(), Value::String(id.to_owned()));
    }
    serde_json::to_vec(&document).map_err(|source| Error::Serialize { source })
}

/// Returns the body of a successful response. `headers` are the per-call
/// headers, already checked by `HeaderBag::finish`.
pub(crate) async fn send(
    client: &Client,
    method: &Method,
    segments: &[String],
    query: &[(String, String)],
    headers: HeaderMap,
    body: Option<&[u8]>,
) -> Result<Vec<u8>> {
    let url = endpoint_url(client.base_url(), segments, query)?;
    let headers = crate::headers::merged(client.default_headers(), headers);

    let mut request = client
        .http()
        .request(method.clone(), url)
        .headers(headers)
        .header(AUTHORIZATION, client.authorization().clone());
    if let Some(body) = body {
        request = request.header(CONTENT_TYPE, JSON_API).body(body.to_vec());
    }

    let response = request
        .send()
        .await
        .map_err(|source| Error::Http { source })?;

    let status = response.status();
    let retry_after = retry_after(response.headers());
    let bytes = response
        .bytes()
        .await
        .map_err(|source| Error::Http { source })?;

    if !status.is_success() {
        return Err(Error::Api {
            status,
            errors: api_errors(&bytes),
            retry_after,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        });
    }

    Ok(bytes.to_vec())
}

/// An empty body decodes as `null`, which [`NoContent`](crate::NoContent)
/// accepts.
pub(crate) fn decode<R: DeserializeOwned>(bytes: &[u8]) -> Result<R> {
    let bytes = if bytes.iter().all(u8::is_ascii_whitespace) {
        b"null".as_slice()
    } else {
        bytes
    };
    serde_json::from_slice(bytes).map_err(|source| Error::Deserialize { source })
}

/// The cursor in `links.next`, or `None` on the last page. A link without a
/// cursor is an error, not the end, so results are never cut short silently.
///
/// Only the cursor is used, so a link to another host never gets the API key.
pub(crate) fn next_cursor(bytes: &[u8], cursor_param: &str, base: &Url) -> Result<Option<String>> {
    #[derive(Deserialize)]
    struct Document {
        #[serde(default)]
        links: Option<Links>,
    }
    #[derive(Deserialize)]
    struct Links {
        #[serde(default)]
        next: Option<String>,
    }

    let document: Document =
        serde_json::from_slice(bytes).map_err(|source| Error::Deserialize { source })?;
    let next = match document.links {
        Some(Links { next: Some(next) }) => next,
        _ => return Ok(None),
    };
    let url = match base.join(&next) {
        Ok(url) => url,
        Err(_) => return Err(Error::InvalidNextPage { link: next }),
    };
    let cursor = url
        .query_pairs()
        .find(|(name, _)| name == cursor_param)
        .map(|(_, value)| value.into_owned());
    match cursor {
        Some(cursor) => Ok(Some(cursor)),
        None => Err(Error::InvalidNextPage { link: next }),
    }
}

fn api_errors(bytes: &[u8]) -> Vec<ApiError> {
    #[derive(Deserialize)]
    struct ErrorDocument {
        #[serde(default)]
        errors: Vec<ApiError>,
    }

    // A body that isn't a JSON:API error document still reaches the caller
    // through `Error::Api::body`.
    match serde_json::from_slice::<ErrorDocument>(bytes) {
        Ok(document) => document.errors,
        Err(_) => Vec::new(),
    }
}

/// Klaviyo sends the delay in seconds.
fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?;
    match value.trim().parse::<u64>() {
        Ok(seconds) => Some(Duration::from_secs(seconds)),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segments(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    #[test]
    fn endpoint_url_keeps_base_prefix() {
        let base = Url::parse("https://example.com/proxy/").expect("valid url");
        let url = endpoint_url(&base, &segments(&["api", "lists"]), &[]).expect("joins");
        assert_eq!(url.path(), "/proxy/api/lists");
    }

    #[test]
    fn endpoint_url_encodes_caller_supplied_segments() {
        let base = Url::parse("https://example.com/").expect("valid url");
        let url =
            endpoint_url(&base, &segments(&["api", "lists", "a/../b?x"]), &[]).expect("joins");
        assert_eq!(url.path(), "/api/lists/a%2F..%2Fb%3Fx");
        assert!(url.query().is_none());
    }

    #[test]
    fn endpoint_url_rejects_segments_url_would_rewrite() {
        let base = Url::parse("https://example.com/").expect("valid url");
        for segment in ["", ".", "..", "..\t", "a\nb"] {
            assert!(
                matches!(
                    endpoint_url(&base, &segments(&["api", "lists", segment]), &[]),
                    Err(Error::InvalidPathSegment { segment: ref s }) if s == segment
                ),
                "{segment:?} should be rejected"
            );
        }
    }

    #[test]
    fn endpoint_url_encodes_json_api_query() {
        let base = Url::parse("https://a.klaviyo.com").expect("valid url");
        let query = vec![
            (
                "filter".to_owned(),
                "equals(messages.channel,'email')".to_owned(),
            ),
            ("fields[campaign]".to_owned(), "name,status".to_owned()),
        ];
        let url = endpoint_url(&base, &segments(&["api", "campaigns"]), &query).expect("joins");
        let pairs: Vec<_> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs, query);
    }

    #[cfg(feature = "_write")]
    #[test]
    fn json_body_with_id_overrides_data_id() {
        let body = crate::types::ListPartialUpdateQuery::default();
        let encoded = json_body(&body, Some("Y6nRLr")).expect("serializes");
        let decoded: serde_json::Value = serde_json::from_slice(&encoded).expect("valid json");
        assert_eq!(decoded["data"]["id"], "Y6nRLr");
        assert_eq!(decoded["data"]["type"], "list");
    }

    #[test]
    fn no_content_accepts_an_empty_or_any_json_body() {
        let bodies: [&[u8]; 3] = [b"", b" \n", br#"{ "data": { "id": "job-1" } }"#];
        for body in bodies {
            decode::<crate::NoContent>(body).expect("NoContent ignores the body");
        }
    }

    fn base() -> Url {
        Url::parse("https://a.klaviyo.com").expect("valid url")
    }

    #[test]
    fn next_cursor_reads_the_cursor_from_links_next() {
        let body = br#"{
            "data": [],
            "links": {
                "self": "https://a.klaviyo.com/api/profiles",
                "next": "https://a.klaviyo.com/api/profiles?page%5Bcursor%5D=bmV4dA&page%5Bsize%5D=20"
            }
        }"#;
        let cursor = next_cursor(body, "page[cursor]", &base()).expect("usable link");
        assert_eq!(cursor.as_deref(), Some("bmV4dA"));
    }

    #[test]
    fn next_cursor_resolves_a_relative_link() {
        let body = br#"{ "data": {}, "links": { "self": "x", "next": "/api/campaign-values-reports?page_cursor=abc" } }"#;
        let cursor = next_cursor(body, "page_cursor", &base()).expect("usable link");
        assert_eq!(cursor.as_deref(), Some("abc"));
    }

    #[test]
    fn next_cursor_is_none_on_the_last_page() {
        let body = br#"{ "data": [], "links": { "self": "https://a.klaviyo.com/api/lists", "next": null } }"#;
        assert!(matches!(
            next_cursor(body, "page[cursor]", &base()),
            Ok(None)
        ));
        assert!(matches!(
            next_cursor(b"{}", "page[cursor]", &base()),
            Ok(None)
        ));
    }

    #[test]
    fn next_link_without_a_cursor_is_an_error() {
        let body = br#"{ "data": [], "links": { "self": "x", "next": "https://a.klaviyo.com/api/lists?offset=10" } }"#;
        assert!(matches!(
            next_cursor(body, "page[cursor]", &base()),
            Err(Error::InvalidNextPage { .. })
        ));
    }

    #[test]
    fn api_errors_are_parsed_from_json_api_body() {
        let body = br#"{ "errors": [ {
            "id": "e1", "code": "invalid", "title": "Invalid input.",
            "detail": "name is required", "source": { "pointer": "/data/attributes/name" }
        } ] }"#;
        let errors = api_errors(body);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "invalid");
        assert_eq!(
            errors[0]
                .source
                .as_ref()
                .and_then(|source| source.pointer.as_deref()),
            Some("/data/attributes/name")
        );
        assert!(api_errors(b"not json").is_empty());
    }

    #[test]
    fn retry_after_parses_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("12"));
        assert_eq!(retry_after(&headers), Some(Duration::from_secs(12)));
        assert_eq!(retry_after(&HeaderMap::new()), None);
    }
}
