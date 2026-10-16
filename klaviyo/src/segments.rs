//! Segments and segment membership.
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
//! let active = client
//!     .segments()
//!     .list()
//!     .filter("equals(is_active,true)")
//!     .send()
//!     .await?;
//! for segment in &active.data {
//!     println!("segment {}", segment.id);
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::Client;
use crate::operations;
use crate::request::RequestBuilder;
use crate::types;

/// Accessor returned by [`Client::segments`](crate::Client::segments).
#[derive(Debug)]
pub struct Segments<'a> {
    client: &'a Client,
}

impl<'a> Segments<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /api/segments` — list segments.
    ///
    /// Filter on `name`, `id`, `created`, `updated`, `is_active`,
    /// `is_starred`. Up to 10 per page. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_segments
    pub fn list(&self) -> ListSegments<'a> {
        RequestBuilder::new(self.client, operations::GetSegments::path())
    }

    /// `GET /api/segments/{id}` — get one segment.
    ///
    /// `additional_fields("segment", &["profile_count"])` adds the member
    /// count, under a much lower rate limit. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_segment
    pub fn get(&self, segment_id: &str) -> GetSegment<'a> {
        RequestBuilder::new(self.client, operations::GetSegment::path(segment_id))
    }

    /// `GET /api/segments/{id}/profiles` — the profiles in a segment.
    ///
    /// Filter on `profile_id`, `email`, `phone_number`, `push_token`,
    /// `joined_group_at`. Up to 100 per page. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_profiles_for_segment
    pub fn profiles(&self, segment_id: &str) -> ListSegmentProfiles<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetProfilesForSegment::path(segment_id),
        )
    }
}

/// Builder for [`Segments::list`].
pub type ListSegments<'a> = RequestBuilder<
    'a,
    types::GetSegmentListResponseCollectionCompoundDocument,
    operations::GetSegments,
>;

/// Builder for [`Segments::get`].
pub type GetSegment<'a> =
    RequestBuilder<'a, types::GetSegmentRetrieveResponseCompoundDocument, operations::GetSegment>;

/// Builder for [`Segments::profiles`].
pub type ListSegmentProfiles<'a> = RequestBuilder<
    'a,
    types::GetSegmentMemberResponseCollection,
    operations::GetProfilesForSegment,
>;
