//! The request builder and page iterator.

use std::fmt;
use std::marker::PhantomData;
use std::mem;

use reqwest::Method;
use serde::de::{DeserializeOwned, IgnoredAny};
use serde::{Deserialize, Deserializer};

use crate::client::Client;
use crate::error::{Error, Result};
use crate::header::{HeaderMap, HeaderName, HeaderValue};
use crate::headers::HeaderBag;
use crate::operations::{
    AdditionalFields, Fields, Filter, Include, Operation, PageSize, Paginated, Path, Sort,
};

/// The response of a `202` / `204` operation. Any body is ignored, so a
/// successful write is never reported as failed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoContent;

impl<'de> Deserialize<'de> for NoContent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        IgnoredAny::deserialize(deserializer)?;
        Ok(NoContent)
    }
}

/// A request to one operation, dispatched with [`send`](Self::send).
///
/// `Op` is the marker from [`operations`](crate::operations). It fixes the
/// method and path, and the query options offered are the ones the
/// operation accepts.
#[must_use = "request builders do nothing until `.send().await` is called"]
pub struct RequestBuilder<'a, Resp, Op> {
    client: &'a Client,
    segments: Vec<String>,
    query: Vec<(String, String)>,
    body: Option<Result<Vec<u8>>>,
    headers: HeaderBag,
    marker: PhantomData<fn() -> (Resp, Op)>,
}

impl<Resp, Op: Operation> fmt::Debug for RequestBuilder<'_, Resp, Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body_len = match &self.body {
            Some(Ok(body)) => Some(body.len()),
            _ => None,
        };
        f.debug_struct("RequestBuilder")
            .field("client", self.client)
            .field("operation", &Op::ID)
            .field("segments", &self.segments)
            .field("query", &self.query)
            .field("body_len", &body_len)
            .field("headers", &self.headers)
            .finish()
    }
}

impl<'a, Resp, Op: Operation> RequestBuilder<'a, Resp, Op> {
    pub(crate) fn new(client: &'a Client, path: Path<'_, Op>) -> Self {
        Self {
            client,
            segments: path.segments().iter().map(|s| (*s).to_owned()).collect(),
            query: Vec::new(),
            body: None,
            headers: HeaderBag::default(),
            marker: PhantomData,
        }
    }

    /// A serialization error surfaces from `send`.
    #[cfg(feature = "_write")]
    pub(crate) fn with_body(client: &'a Client, path: Path<'_, Op>, body: Result<Vec<u8>>) -> Self {
        let mut builder = Self::new(client, path);
        builder.body = Some(body);
        builder
    }

    /// Replaces any earlier value of `name`.
    pub(crate) fn query(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        self.query.retain(|(existing, _)| *existing != name);
        self.query.push((name, value.into()));
        self
    }

    /// Adds a header, overriding a client default of the same name. An
    /// invalid or reserved header is reported as [`Error::InvalidHeader`] by
    /// [`send`](Self::send).
    pub fn header<N, V>(mut self, name: N, value: V) -> Self
    where
        N: TryInto<HeaderName> + AsRef<str>,
        V: TryInto<HeaderValue>,
    {
        self.headers.add(name, value);
        self
    }

    /// Merges `headers` like `reqwest::RequestBuilder::headers`: names in
    /// `headers` replace earlier values, others are kept.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.merge(headers);
        self
    }
}

impl<Resp, Op: Fields> RequestBuilder<'_, Resp, Op> {
    /// `fields[<resource_type>]`. See [sparse fieldsets][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/api_overview#sparse-fieldsets
    pub fn fields(self, resource_type: &str, fields: &[&str]) -> Self {
        self.query(format!("fields[{resource_type}]"), fields.join(","))
    }
}

impl<Resp, Op: AdditionalFields> RequestBuilder<'_, Resp, Op> {
    /// `additional-fields[<resource_type>]`, e.g. profile `subscriptions` or
    /// list `profile_count`.
    pub fn additional_fields(self, resource_type: &str, fields: &[&str]) -> Self {
        self.query(
            format!("additional-fields[{resource_type}]"),
            fields.join(","),
        )
    }
}

impl<Resp, Op: Include> RequestBuilder<'_, Resp, Op> {
    /// Side-loads related resources into `included`. See
    /// [relationships][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/api_overview#relationships
    pub fn include(self, relationships: &[&str]) -> Self {
        self.query("include", relationships.join(","))
    }
}

impl<Resp, Op: Filter> RequestBuilder<'_, Resp, Op> {
    /// A filter expression such as `equals(status,'Draft')`. See
    /// [filtering][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/api_overview#filtering
    pub fn filter(self, expression: &str) -> Self {
        self.query("filter", expression)
    }
}

impl<Resp, Op: Sort> RequestBuilder<'_, Resp, Op> {
    /// A field to sort by; `-` prefix for descending.
    pub fn sort(self, field: &str) -> Self {
        self.query("sort", field)
    }
}

impl<Resp, Op: PageSize> RequestBuilder<'_, Resp, Op> {
    /// Results per page, up to the endpoint's maximum.
    pub fn page_size(self, size: u32) -> Self {
        self.query("page[size]", size.to_string())
    }
}

impl<'a, Resp, Op: Paginated> RequestBuilder<'a, Resp, Op> {
    /// Start from `cursor` instead of the first page.
    pub fn page_cursor(self, cursor: &str) -> Self {
        self.query(Op::CURSOR_PARAM, cursor)
    }

    /// Iterates over every page.
    ///
    /// ```no_run
    /// # #[cfg(feature = "lists")]
    /// # async fn run(client: klaviyo::Client) -> klaviyo::Result<()> {
    /// let mut pages = client.lists().list().pages();
    /// while let Some(page) = pages.next_page().await {
    ///     for list in page?.data {
    ///         println!("{}", list.id);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn pages(self) -> Pages<'a, Resp> {
        let (headers, body, state) = match (self.headers.finish(), self.body.transpose()) {
            (Ok(headers), Ok(body)) => (headers, body, PageState::First),
            (Err(error), _) | (_, Err(error)) => (HeaderMap::new(), None, PageState::Broken(error)),
        };
        Pages {
            client: self.client,
            method: Op::METHOD,
            segments: self.segments,
            query: self.query,
            headers,
            body,
            cursor_param: Op::CURSOR_PARAM,
            state,
            marker: PhantomData,
        }
    }
}

impl<Resp, Op> RequestBuilder<'_, Resp, Op>
where
    Resp: DeserializeOwned,
    Op: Operation,
{
    /// Sends the request.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn send(self) -> Result<Resp> {
        let headers = self.headers.finish()?;
        let body = self.body.transpose()?;
        let method = Op::METHOD;
        let bytes = crate::http::send(
            self.client,
            &method,
            &self.segments,
            &self.query,
            headers,
            body.as_deref(),
        )
        .await?;
        crate::http::decode(&bytes)
    }
}

#[derive(Debug)]
enum PageState {
    First,
    Next(String),
    Broken(Error),
    Done,
}

/// Pages of a collection or report, from [`RequestBuilder::pages`].
///
/// A failed request leaves the position unchanged, so calling
/// [`next_page`](Self::next_page) again retries the same page. A
/// `links.next` that can't be followed is reported as
/// [`Error::InvalidNextPage`] after its page.
#[must_use = "pages do nothing until `.next_page().await` is called"]
pub struct Pages<'a, Resp> {
    client: &'a Client,
    method: Method,
    segments: Vec<String>,
    query: Vec<(String, String)>,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    cursor_param: &'static str,
    state: PageState,
    marker: PhantomData<fn() -> Resp>,
}

impl<Resp> fmt::Debug for Pages<'_, Resp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pages")
            .field("client", self.client)
            .field("method", &self.method)
            .field("segments", &self.segments)
            .field("query", &self.query)
            .field("state", &self.state)
            .finish()
    }
}

impl<Resp: DeserializeOwned> Pages<'_, Resp> {
    /// The next page, or `None` after the last one.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn next_page(&mut self) -> Option<Result<Resp>> {
        let current = match mem::replace(&mut self.state, PageState::Done) {
            PageState::Done => return None,
            PageState::Broken(error) => return Some(Err(error)),
            PageState::First => None,
            PageState::Next(cursor) => Some(cursor),
        };

        let mut query = self.query.clone();
        if let Some(cursor) = &current {
            query.retain(|(name, _)| name != self.cursor_param);
            query.push((self.cursor_param.to_owned(), cursor.clone()));
        }

        let bytes = match crate::http::send(
            self.client,
            &self.method,
            &self.segments,
            &query,
            self.headers.clone(),
            self.body.as_deref(),
        )
        .await
        {
            Ok(bytes) => bytes,
            Err(error) => {
                self.state = match current {
                    Some(cursor) => PageState::Next(cursor),
                    None => PageState::First,
                };
                return Some(Err(error));
            }
        };

        let page = match crate::http::decode(&bytes) {
            Ok(page) => page,
            Err(error) => return Some(Err(error)),
        };

        self.state =
            match crate::http::next_cursor(&bytes, self.cursor_param, self.client.base_url()) {
                Ok(None) => PageState::Done,
                Ok(Some(next)) if Some(&next) == current.as_ref() => {
                    PageState::Broken(Error::InvalidNextPage { link: next })
                }
                Ok(Some(next)) => PageState::Next(next),
                Err(error) => PageState::Broken(error),
            };
        Some(Ok(page))
    }
}
