//! Lists and list membership.
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
//! let newsletter = client
//!     .lists()
//!     .list()
//!     .filter("equals(name,'Newsletter')")
//!     .send()
//!     .await?;
//!
//! if let Some(list) = newsletter.data.first() {
//!     client
//!         .lists()
//!         .add_profiles(&list.id, &["01GDDKASAP8TKDDA2GRZDSVP4H"])
//!         .send()
//!         .await?;
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::Client;
use crate::http::json_body;
use crate::operations;
use crate::request::{NoContent, RequestBuilder};
use crate::types;

/// Accessor returned by [`Client::lists`](crate::Client::lists).
#[derive(Debug)]
pub struct Lists<'a> {
    client: &'a Client,
}

impl<'a> Lists<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /api/lists` — list lists.
    ///
    /// Filter on `name`, `id`, `created`, `updated`. Up to 10 per page. See
    /// the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_lists
    pub fn list(&self) -> ListLists<'a> {
        RequestBuilder::new(self.client, operations::GetLists::path())
    }

    /// `GET /api/lists/{id}` — get one list.
    ///
    /// `additional_fields("list", &["profile_count"])` adds the member count,
    /// under a much lower rate limit. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_list
    pub fn get(&self, list_id: &str) -> GetList<'a> {
        RequestBuilder::new(self.client, operations::GetList::path(list_id))
    }

    /// `POST /api/lists` — create a list.
    ///
    /// Limited to 150 a day per account. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/create_list
    pub fn create(&self, request: &types::ListCreateQuery) -> CreateList<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CreateList::path(),
            json_body(request, None),
        )
    }

    /// `PATCH /api/lists/{id}` — rename a list.
    ///
    /// `data.id` in the body is set to `list_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/update_list
    pub fn update(&self, list_id: &str, request: &types::ListPartialUpdateQuery) -> UpdateList<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::UpdateList::path(list_id),
            json_body(request, Some(list_id)),
        )
    }

    /// `DELETE /api/lists/{id}` — delete a list.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/delete_list
    pub fn delete(&self, list_id: &str) -> DeleteList<'a> {
        RequestBuilder::new(self.client, operations::DeleteList::path(list_id))
    }

    /// `GET /api/lists/{id}/profiles` — the profiles in a list.
    ///
    /// Filter on `email`, `phone_number`, `push_token`, `joined_group_at`. Up
    /// to 100 per page. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_profiles_for_list
    pub fn profiles(&self, list_id: &str) -> ListListProfiles<'a> {
        RequestBuilder::new(self.client, operations::GetProfilesForList::path(list_id))
    }

    /// `POST /api/lists/{id}/relationships/profiles` — add profiles to a
    /// list, by profile ID.
    ///
    /// This doesn't change consent; use `Profiles::subscribe` (the `profiles`
    /// feature) for that. Up to 1,000 profiles per call. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/add_profiles_to_list
    pub fn add_profiles(&self, list_id: &str, profile_ids: &[&str]) -> AddProfilesToList<'a> {
        let request = types::ListMembersAddQuery {
            data: profile_ids
                .iter()
                .map(|id| types::ListMembersAddQueryDataItem {
                    id: (*id).to_owned(),
                    type_: Default::default(),
                })
                .collect(),
        };
        RequestBuilder::with_body(
            self.client,
            operations::AddProfilesToList::path(list_id),
            json_body(&request, None),
        )
    }

    /// `DELETE /api/lists/{id}/relationships/profiles` — remove profiles from
    /// a list, by profile ID.
    ///
    /// This doesn't unsubscribe them. Up to 1,000 profiles per call. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/remove_profiles_from_list
    pub fn remove_profiles(
        &self,
        list_id: &str,
        profile_ids: &[&str],
    ) -> RemoveProfilesFromList<'a> {
        let request = types::ListMembersDeleteQuery {
            data: profile_ids
                .iter()
                .map(|id| types::ListMembersDeleteQueryDataItem {
                    id: (*id).to_owned(),
                    type_: Default::default(),
                })
                .collect(),
        };
        RequestBuilder::with_body(
            self.client,
            operations::RemoveProfilesFromList::path(list_id),
            json_body(&request, None),
        )
    }
}

/// Builder for [`Lists::list`].
pub type ListLists<'a> =
    RequestBuilder<'a, types::GetListListResponseCollectionCompoundDocument, operations::GetLists>;

/// Builder for [`Lists::get`].
pub type GetList<'a> =
    RequestBuilder<'a, types::GetListRetrieveResponseCompoundDocument, operations::GetList>;

/// Builder for [`Lists::create`].
pub type CreateList<'a> = RequestBuilder<'a, types::PostListCreateResponse, operations::CreateList>;

/// Builder for [`Lists::update`].
pub type UpdateList<'a> =
    RequestBuilder<'a, types::PatchListPartialUpdateResponse, operations::UpdateList>;

/// Builder for [`Lists::delete`].
pub type DeleteList<'a> = RequestBuilder<'a, NoContent, operations::DeleteList>;

/// Builder for [`Lists::profiles`].
pub type ListListProfiles<'a> =
    RequestBuilder<'a, types::GetListMemberResponseCollection, operations::GetProfilesForList>;

/// Builder for [`Lists::add_profiles`].
pub type AddProfilesToList<'a> = RequestBuilder<'a, NoContent, operations::AddProfilesToList>;

/// Builder for [`Lists::remove_profiles`].
pub type RemoveProfilesFromList<'a> =
    RequestBuilder<'a, NoContent, operations::RemoveProfilesFromList>;
