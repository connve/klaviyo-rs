//! Profiles, bulk import, subscriptions, and suppressions.
//!
//! Consent is separate from profile data: only [`Profiles::subscribe`]
//! records marketing consent.
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
//! let mut pages = client
//!     .profiles()
//!     .list()
//!     .filter("greater-than(updated,2026-10-01T00:00:00Z)")
//!     .page_size(100)
//!     .pages();
//! while let Some(page) = pages.next_page().await {
//!     for profile in page?.data {
//!         println!("profile {:?}", profile.id);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::Client;
use crate::http::json_body;
use crate::operations;
use crate::request::{NoContent, RequestBuilder};
use crate::types;

/// Accessor returned by [`Client::profiles`](crate::Client::profiles).
#[derive(Debug)]
pub struct Profiles<'a> {
    client: &'a Client,
}

impl<'a> Profiles<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /api/profiles` — list profiles.
    ///
    /// Filter on `id`, `email`, `phone_number`, `external_id`, `created`,
    /// `updated`, and suppression fields. Up to 100 per page.
    /// `additional_fields("profile", &["subscriptions"])` adds consent
    /// details. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_profiles
    pub fn list(&self) -> ListProfiles<'a> {
        RequestBuilder::new(self.client, operations::GetProfiles::path())
    }

    /// `GET /api/profiles/{id}` — get one profile.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_profile
    pub fn get(&self, profile_id: &str) -> GetProfile<'a> {
        RequestBuilder::new(self.client, operations::GetProfile::path(profile_id))
    }

    /// `POST /api/profiles` — create a profile.
    ///
    /// Fails with 409 if a profile with the same identifiers exists; use
    /// [`upsert`](Self::upsert) to create or update. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/create_profile
    pub fn create(&self, request: &types::ProfileCreateQuery) -> CreateProfile<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CreateProfile::path(),
            json_body(request, None),
        )
    }

    /// `PATCH /api/profiles/{id}` — update a profile.
    ///
    /// A field set to `null` is cleared; a field left out is unchanged.
    /// `data.id` in the body is set to `profile_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/update_profile
    pub fn update(
        &self,
        profile_id: &str,
        request: &types::ProfilePartialUpdateQuery,
    ) -> UpdateProfile<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::UpdateProfile::path(profile_id),
            json_body(request, Some(profile_id)),
        )
    }

    /// `POST /api/profile-import` — create a profile, or update the one
    /// matching its identifiers.
    ///
    /// Klaviyo answers 201 for a new profile and 200 for an update. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/create_or_update_profile
    pub fn upsert(&self, request: &types::ProfileUpsertQuery) -> UpsertProfile<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CreateOrUpdateProfile::path(),
            json_body(request, None),
        )
    }

    /// `POST /api/profile-bulk-import-jobs` — start a job that creates or
    /// updates up to 10,000 profiles, optionally adding them to lists.
    ///
    /// Poll [`get_bulk_import_job`](Self::get_bulk_import_job) and read
    /// failures with [`bulk_import_errors`](Self::bulk_import_errors). See
    /// the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/bulk_import_profiles
    pub fn bulk_import(&self, request: &types::ProfileImportJobCreateQuery) -> BulkImport<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::BulkImportProfiles::path(),
            json_body(request, None),
        )
    }

    /// `GET /api/profile-bulk-import-jobs/{id}` — status of a bulk import
    /// job.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_bulk_import_profiles_job
    pub fn get_bulk_import_job(&self, job_id: &str) -> GetBulkImportJob<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetBulkImportProfilesJob::path(job_id),
        )
    }

    /// `GET /api/profile-bulk-import-jobs/{id}/import-errors` — the profiles
    /// a bulk import job rejected, and why.
    ///
    /// Up to 100 per page. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_errors_for_bulk_import_profiles_job
    pub fn bulk_import_errors(&self, job_id: &str) -> ListBulkImportErrors<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetErrorsForBulkImportProfilesJob::path(job_id),
        )
    }

    /// `POST /api/profile-subscription-bulk-create-jobs` — subscribe up to
    /// 1,000 profiles to email, SMS, WhatsApp, or push marketing.
    ///
    /// Lists with double opt-in send a confirmation first. This also lifts
    /// unsubscribe and manual suppressions. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/bulk_subscribe_profiles
    pub fn subscribe(&self, request: &types::SubscriptionCreateJobCreateQuery) -> Subscribe<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::BulkSubscribeProfiles::path(),
            json_body(request, None),
        )
    }

    /// `POST /api/profile-subscription-bulk-delete-jobs` — unsubscribe up to
    /// 100 profiles.
    ///
    /// Profiles that aren't in the given list are unsubscribed globally:
    /// check list membership first. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/bulk_unsubscribe_profiles
    pub fn unsubscribe(
        &self,
        request: &types::SubscriptionDeleteJobCreateQuery,
    ) -> Unsubscribe<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::BulkUnsubscribeProfiles::path(),
            json_body(request, None),
        )
    }

    /// `POST /api/profile-suppression-bulk-create-jobs` — suppress profiles
    /// from email marketing, by email or by list / segment.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/bulk_suppress_profiles
    pub fn suppress(&self, request: &types::SuppressionCreateJobCreateQuery) -> Suppress<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::BulkSuppressProfiles::path(),
            json_body(request, None),
        )
    }

    /// `POST /api/profile-suppression-bulk-delete-jobs` — lift manual
    /// (`USER_SUPPRESSED`) suppressions.
    ///
    /// Unsubscribes, hard bounces, and invalid emails stay suppressed. See
    /// the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/bulk_unsuppress_profiles
    pub fn unsuppress(&self, request: &types::SuppressionDeleteJobCreateQuery) -> Unsuppress<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::BulkUnsuppressProfiles::path(),
            json_body(request, None),
        )
    }
}

/// Builder for [`Profiles::list`].
pub type ListProfiles<'a> = RequestBuilder<
    'a,
    types::GetProfileResponsePluralConversationsCollectionCompoundDocument,
    operations::GetProfiles,
>;

/// Builder for [`Profiles::get`].
pub type GetProfile<'a> = RequestBuilder<
    'a,
    types::GetProfileResponsePluralConversationsCompoundDocument,
    operations::GetProfile,
>;

/// Builder for [`Profiles::create`].
pub type CreateProfile<'a> =
    RequestBuilder<'a, types::PostProfileResponse, operations::CreateProfile>;

/// Builder for [`Profiles::update`].
pub type UpdateProfile<'a> =
    RequestBuilder<'a, types::PatchProfileResponse, operations::UpdateProfile>;

/// Builder for [`Profiles::upsert`].
pub type UpsertProfile<'a> =
    RequestBuilder<'a, types::PostProfileResponse, operations::CreateOrUpdateProfile>;

/// Builder for [`Profiles::bulk_import`].
pub type BulkImport<'a> =
    RequestBuilder<'a, types::PostProfileImportJobResponse, operations::BulkImportProfiles>;

/// Builder for [`Profiles::get_bulk_import_job`].
pub type GetBulkImportJob<'a> = RequestBuilder<
    'a,
    types::GetProfileImportJobResponseCompoundDocument,
    operations::GetBulkImportProfilesJob,
>;

/// Builder for [`Profiles::bulk_import_errors`].
pub type ListBulkImportErrors<'a> = RequestBuilder<
    'a,
    types::GetImportErrorResponseCollection,
    operations::GetErrorsForBulkImportProfilesJob,
>;

/// Builder for [`Profiles::subscribe`].
pub type Subscribe<'a> = RequestBuilder<'a, NoContent, operations::BulkSubscribeProfiles>;

/// Builder for [`Profiles::unsubscribe`].
pub type Unsubscribe<'a> = RequestBuilder<'a, NoContent, operations::BulkUnsubscribeProfiles>;

/// Builder for [`Profiles::suppress`].
pub type Suppress<'a> = RequestBuilder<
    'a,
    types::PostBulkProfileSuppressionsCreateJobResponse,
    operations::BulkSuppressProfiles,
>;

/// Builder for [`Profiles::unsuppress`].
pub type Unsuppress<'a> = RequestBuilder<
    'a,
    types::PostBulkProfileSuppressionsRemoveJobResponse,
    operations::BulkUnsuppressProfiles,
>;
