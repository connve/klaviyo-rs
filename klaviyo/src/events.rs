//! Server-side event tracking and event reads.
//!
//! Creating an event also creates or updates its profile. Success means the
//! event was queued, not processed.
//!
//! ```no_run
//! use klaviyo::Client;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .api_key(std::env::var("KLAVIYO_API_KEY")?)
//!     .build()?;
//!
//! let recent = client
//!     .events()
//!     .list()
//!     .filter("greater-than(datetime,2026-10-01T00:00:00Z)")
//!     .sort("-datetime")
//!     .send()
//!     .await?;
//! for event in &recent.data {
//!     println!("event {}", event.id);
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::Client;
use crate::http::json_body;
use crate::operations;
use crate::request::{NoContent, RequestBuilder};
use crate::types;

/// Accessor returned by [`Client::events`](crate::Client::events).
#[derive(Debug)]
pub struct Events<'a> {
    client: &'a Client,
}

impl<'a> Events<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /api/events` — list events.
    ///
    /// Filter on `metric_id`, `profile_id`, `datetime`, `timestamp`. Up to
    /// 1,000 per page. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_events
    pub fn list(&self) -> ListEvents<'a> {
        RequestBuilder::new(self.client, operations::GetEvents::path())
    }

    /// `GET /api/events/{id}` — get one event.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_event
    pub fn get(&self, event_id: &str) -> GetEvent<'a> {
        RequestBuilder::new(self.client, operations::GetEvent::path(event_id))
    }

    /// `POST /api/events` — track one event for a profile.
    ///
    /// The profile needs at least one identifier and the metric a `name`.
    /// Not idempotent unless you set `unique_id`: a retry can record the
    /// event twice. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/create_event
    pub fn create(&self, request: &types::EventCreateQueryV2) -> CreateEvent<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CreateEvent::path(),
            json_body(request, None),
        )
    }

    /// `POST /api/event-bulk-create-jobs` — track up to 1,000 events across
    /// profiles in one request (5 MB max).
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/bulk_create_events
    pub fn bulk_create(&self, request: &types::EventsBulkCreateJob) -> BulkCreateEvents<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::BulkCreateEvents::path(),
            json_body(request, None),
        )
    }
}

/// Builder for [`Events::list`].
pub type ListEvents<'a> =
    RequestBuilder<'a, types::GetEventResponseCollectionCompoundDocument, operations::GetEvents>;

/// Builder for [`Events::get`].
pub type GetEvent<'a> =
    RequestBuilder<'a, types::GetEventResponseCompoundDocument, operations::GetEvent>;

/// Builder for [`Events::create`].
pub type CreateEvent<'a> = RequestBuilder<'a, NoContent, operations::CreateEvent>;

/// Builder for [`Events::bulk_create`].
pub type BulkCreateEvents<'a> = RequestBuilder<'a, NoContent, operations::BulkCreateEvents>;
