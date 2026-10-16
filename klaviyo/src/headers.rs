//! Request header plumbing shared by the client builder and every request
//! builder.

#[cfg(feature = "_api")]
use reqwest::header::HeaderValue;
use reqwest::header::{
    HeaderMap, HeaderName, ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, HOST,
    TRANSFER_ENCODING, USER_AGENT,
};

use crate::error::{Error, Result};

/// Pins the API contract the types were generated for, so it's reserved.
pub(crate) const REVISION: &str = "revision";

/// Headers the SDK or the HTTP stack sets; overriding them would break or
/// misroute requests.
const FORBIDDEN_HEADERS: &[HeaderName] = &[
    AUTHORIZATION,
    CONTENT_TYPE,
    CONTENT_LENGTH,
    TRANSFER_ENCODING,
    ACCEPT,
    HOST,
    USER_AGENT,
];

fn is_forbidden(name: &HeaderName) -> bool {
    FORBIDDEN_HEADERS.contains(name) || name.as_str() == REVISION
}

/// Returns [`Error::InvalidHeader`] for the first reserved header in `headers`.
pub(crate) fn reject_forbidden(headers: &HeaderMap) -> Result<()> {
    match headers.keys().find(|name| is_forbidden(name)) {
        Some(name) => Err(Error::InvalidHeader {
            name: name.as_str().to_owned(),
        }),
        None => Ok(()),
    }
}

/// `reqwest::RequestBuilder::headers` semantics: names in `incoming` replace
/// existing values, other names are kept.
#[cfg(feature = "_api")]
fn replace_names(target: &mut HeaderMap, incoming: HeaderMap) {
    for name in incoming.keys() {
        target.remove(name);
    }
    for (name, value) in &incoming {
        target.append(name.clone(), value.clone());
    }
}

#[cfg(feature = "_api")]
pub(crate) fn merged(defaults: &HeaderMap, per_call: HeaderMap) -> HeaderMap {
    let mut headers = defaults.clone();
    replace_names(&mut headers, per_call);
    headers
}

/// Per-call headers. Errors surface from `.send()`, so chaining stays
/// infallible.
#[cfg(feature = "_api")]
#[derive(Debug, Default, Clone)]
pub(crate) struct HeaderBag {
    headers: HeaderMap,
    invalid: Option<String>,
}

#[cfg(feature = "_api")]
impl HeaderBag {
    pub(crate) fn merge(&mut self, headers: HeaderMap) {
        replace_names(&mut self.headers, headers);
    }

    pub(crate) fn add<N, V>(&mut self, name: N, value: V)
    where
        N: TryInto<HeaderName> + AsRef<str>,
        V: TryInto<HeaderValue>,
    {
        if self.invalid.is_some() {
            return;
        }
        let raw_name = name.as_ref().to_owned();
        match name.try_into() {
            Err(_) => self.invalid = Some(raw_name),
            Ok(name) => match value.try_into() {
                Err(_) => self.invalid = Some(name.as_str().to_owned()),
                Ok(value) => {
                    self.headers.append(name, value);
                }
            },
        }
    }

    pub(crate) fn finish(self) -> Result<HeaderMap> {
        if let Some(name) = self.invalid {
            return Err(Error::InvalidHeader { name });
        }
        reject_forbidden(&self.headers)?;
        Ok(self.headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_forbidden_flags_reserved_names() {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "text/plain".parse().expect("valid value"));
        assert!(matches!(
            reject_forbidden(&headers),
            Err(Error::InvalidHeader { ref name }) if name == "accept"
        ));
    }

    #[test]
    fn revision_is_reserved() {
        let mut headers = HeaderMap::new();
        headers.insert(REVISION, "2023-01-01".parse().expect("valid value"));
        assert!(matches!(
            reject_forbidden(&headers),
            Err(Error::InvalidHeader { ref name }) if name == "revision"
        ));
    }
}

#[cfg(all(test, feature = "_api"))]
mod bag_tests {
    use super::*;

    #[test]
    fn rejects_forbidden() {
        let mut bag = HeaderBag::default();
        bag.add("Authorization", "Klaviyo-API-Key x");
        assert!(matches!(
            bag.finish(),
            Err(Error::InvalidHeader { ref name }) if name == "authorization"
        ));
    }

    #[test]
    fn rejects_forbidden_in_merged_map() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        let mut bag = HeaderBag::default();
        bag.merge(headers);
        assert!(matches!(
            bag.finish(),
            Err(Error::InvalidHeader { ref name }) if name == "content-type"
        ));
    }

    #[test]
    fn merge_replaces_same_name_and_keeps_others() {
        let mut bag = HeaderBag::default();
        bag.add("X-Request-Id", "first");
        bag.add("X-Request-Id", "second");
        bag.add("X-Tenant", "acme");

        let mut incoming = HeaderMap::new();
        incoming.insert("x-request-id", HeaderValue::from_static("merged"));
        bag.merge(incoming);

        let map = bag.finish().expect("valid headers");
        let request_ids: Vec<_> = map.get_all("x-request-id").iter().collect();
        assert_eq!(request_ids, [HeaderValue::from_static("merged")]);
        assert_eq!(map.get("x-tenant"), Some(&HeaderValue::from_static("acme")));
    }

    #[test]
    fn rejects_invalid_value() {
        let mut bag = HeaderBag::default();
        bag.add("X-Request-Id", "bad\nvalue");
        assert!(matches!(
            bag.finish(),
            Err(Error::InvalidHeader { ref name }) if name == "x-request-id"
        ));
    }

    #[test]
    fn rejects_invalid_name_and_reports_it() {
        let mut bag = HeaderBag::default();
        bag.add("bad header", "x");
        assert!(matches!(
            bag.finish(),
            Err(Error::InvalidHeader { ref name }) if name == "bad header"
        ));
    }

    #[test]
    fn first_invalid_header_is_reported() {
        let mut bag = HeaderBag::default();
        bag.add("X-Request-Id", "bad\nvalue");
        bag.add("X-Trace-Id", "bad\nvalue");
        assert!(matches!(
            bag.finish(),
            Err(Error::InvalidHeader { ref name }) if name == "x-request-id"
        ));
    }

    #[test]
    fn merged_keeps_all_values_of_a_multi_valued_default() {
        let mut defaults = HeaderMap::new();
        defaults.append("x-forwarded-for", HeaderValue::from_static("10.0.0.1"));
        defaults.append("x-forwarded-for", HeaderValue::from_static("10.0.0.2"));
        defaults.insert("x-tenant", HeaderValue::from_static("default"));

        let mut per_call = HeaderMap::new();
        per_call.insert("x-tenant", HeaderValue::from_static("per-call"));

        let headers = merged(&defaults, per_call);
        assert_eq!(headers.get_all("x-forwarded-for").iter().count(), 2);
        assert_eq!(
            headers.get("x-tenant"),
            Some(&HeaderValue::from_static("per-call"))
        );
    }
}
