//! Campaigns, campaign messages and variations, send jobs, and recipient
//! estimates.
//!
//! A campaign is created as a draft with its audiences, messages, and
//! variations; a variation holds the subject, sender, and template. Sends
//! can't be undone.
//!
//! ```no_run
//! use klaviyo::{types, Client};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .api_key(std::env::var("KLAVIYO_API_KEY")?)
//!     .build()?;
//!
//! let campaigns = client
//!     .campaigns()
//!     .list()
//!     .filter("equals(archived,false)")
//!     .sort("-created_at")
//!     .send()
//!     .await?;
//!
//! for campaign in &campaigns.data {
//!     println!("campaign {}", campaign.id);
//! }
//!
//! // Point the first variation of a campaign's first message at a template.
//! let campaign_id = "01JABCDEF";
//! let messages = client.campaigns().messages(campaign_id).send().await?;
//! if let Some(message) = messages.data.first() {
//!     let variations = client.campaigns().variations(&message.id).send().await?;
//!     if let Some(variation) = variations.data.first() {
//!         let request = types::VariationPartialUpdateQuery {
//!             data: types::VariationPartialUpdateQueryResourceObject {
//!                 attributes: types::VariationPartialUpdateQueryResourceObjectAttributes {
//!                     definition: types::VariationDefinition {
//!                         details: Some(
//!                             types::EmailChannelDetails {
//!                                 channel: Some(types::EmailChannelDetailsChannel::Email),
//!                                 template_id: Some("RipRmi".to_owned()),
//!                                 ..Default::default()
//!                             }
//!                             .into(),
//!                         ),
//!                         ..Default::default()
//!                     },
//!                 },
//!                 ..Default::default()
//!             },
//!         };
//!         client
//!             .campaigns()
//!             .update_variation(&variation.id, &request)
//!             .send()
//!             .await?;
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

/// Accessor returned by [`Client::campaigns`](crate::Client::campaigns).
#[derive(Debug)]
pub struct Campaigns<'a> {
    client: &'a Client,
}

impl<'a> Campaigns<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /api/campaigns` — list campaigns.
    ///
    /// Up to 100 per page. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_campaigns
    pub fn list(&self) -> ListCampaigns<'a> {
        RequestBuilder::new(self.client, operations::GetCampaigns::path())
    }

    /// `GET /api/campaigns/{id}` — get one campaign.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_campaign
    pub fn get(&self, campaign_id: &str) -> GetCampaign<'a> {
        RequestBuilder::new(self.client, operations::GetCampaign::path(campaign_id))
    }

    /// `POST /api/campaigns` — create a draft campaign with its audiences,
    /// messages, and variations.
    ///
    /// Set `channel` in each variation's `details`: Klaviyo reads details
    /// without one as email. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/create_campaign
    pub fn create(&self, request: &types::CampaignCreateQuery) -> CreateCampaign<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CreateCampaign::path(),
            json_body(request, None),
        )
    }

    /// `PATCH /api/campaigns/{id}` — update a campaign's definition: name,
    /// archived flag, and send settings.
    ///
    /// `data.id` in the body is set to `campaign_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/update_campaign
    pub fn update(
        &self,
        campaign_id: &str,
        request: &types::CampaignPartialUpdateQuery,
    ) -> UpdateCampaign<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::UpdateCampaign::path(campaign_id),
            json_body(request, Some(campaign_id)),
        )
    }

    /// `DELETE /api/campaigns/{id}` — delete a campaign.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/delete_campaign
    pub fn delete(&self, campaign_id: &str) -> DeleteCampaign<'a> {
        RequestBuilder::new(self.client, operations::DeleteCampaign::path(campaign_id))
    }

    /// `POST /api/campaign-clone` — clone a campaign within the same account.
    ///
    /// `data.id` in the body is set to `campaign_id`, the campaign to copy.
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/clone_campaign
    pub fn clone_campaign(
        &self,
        campaign_id: &str,
        request: &types::CampaignCloneQuery,
    ) -> CloneCampaign<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CloneCampaign::path(),
            json_body(request, Some(campaign_id)),
        )
    }

    /// `GET /api/campaigns/{id}/campaign-messages` — the messages of a
    /// campaign.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_messages_for_campaign
    pub fn messages(&self, campaign_id: &str) -> ListCampaignMessages<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetMessagesForCampaign::path(campaign_id),
        )
    }

    /// `GET /api/campaign-messages/{id}` — get one campaign message.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_campaign_message
    pub fn get_message(&self, message_id: &str) -> GetCampaignMessage<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetCampaignMessage::path(message_id),
        )
    }

    /// `PATCH /api/campaign-messages/{id}` — update a message's name,
    /// tracking and send options, and A/B test settings.
    ///
    /// The subject line, preview text, and sender belong to the message's
    /// variations, not the message. `data.id` in the body is set to
    /// `message_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/update_campaign_message
    pub fn update_message(
        &self,
        message_id: &str,
        request: &types::MessagePartialUpdateQuery,
    ) -> UpdateCampaignMessage<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::UpdateCampaignMessage::path(message_id),
            json_body(request, Some(message_id)),
        )
    }

    /// `GET /api/campaign-messages/{id}/campaign-variations` — the variations
    /// of a campaign message.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_variations_for_campaign_message
    pub fn variations(&self, message_id: &str) -> ListCampaignVariations<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetVariationsForCampaignMessage::path(message_id),
        )
    }

    /// `PATCH /api/campaign-variations/{id}` — update a variation's name and
    /// channel details, such as the subject line, sender, and template.
    ///
    /// `data.id` in the body is set to `variation_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/update_campaign_variation
    pub fn update_variation(
        &self,
        variation_id: &str,
        request: &types::VariationPartialUpdateQuery,
    ) -> UpdateCampaignVariation<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::UpdateCampaignVariation::path(variation_id),
            json_body(request, Some(variation_id)),
        )
    }

    /// `POST /api/campaign-send-jobs` — send or schedule a campaign, as set
    /// by its send strategy.
    ///
    /// Delivers real messages; a retry after a timeout or 5xx can send twice.
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/send_campaign
    pub fn send(&self, campaign_id: &str) -> SendCampaign<'a> {
        let request = types::CampaignSendJobCreateQuery {
            data: types::CampaignSendJobCreateQueryResourceObject {
                id: campaign_id.to_owned(),
                type_: Default::default(),
            },
        };
        RequestBuilder::with_body(
            self.client,
            operations::SendCampaign::path(),
            json_body(&request, None),
        )
    }

    /// `GET /api/campaign-send-jobs/{id}` — status of a campaign's send job.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_campaign_send_job
    pub fn get_send_job(&self, campaign_id: &str) -> GetCampaignSendJob<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetCampaignSendJob::path(campaign_id),
        )
    }

    /// `PATCH /api/campaign-send-jobs/{id}` — cancel a send permanently, or
    /// revert the campaign to a draft, depending on `attributes.action`.
    ///
    /// `data.id` in the body is set to `campaign_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/cancel_campaign_send
    pub fn cancel_send(
        &self,
        campaign_id: &str,
        request: &types::CampaignSendJobPartialUpdateQuery,
    ) -> CancelCampaignSend<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CancelCampaignSend::path(campaign_id),
            json_body(request, Some(campaign_id)),
        )
    }

    /// `POST /api/campaign-recipient-estimation-jobs` — start recounting a
    /// campaign's estimated recipients.
    ///
    /// Poll [`get_recipient_estimation_job`](Self::get_recipient_estimation_job),
    /// then read [`get_recipient_estimation`](Self::get_recipient_estimation).
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/refresh_campaign_recipient_estimation
    pub fn refresh_recipient_estimation(
        &self,
        campaign_id: &str,
    ) -> RefreshRecipientEstimation<'a> {
        let request = types::CampaignRecipientEstimationJobCreateQuery {
            data: types::CampaignRecipientEstimationJobCreateQueryResourceObject {
                id: campaign_id.to_owned(),
                type_: Default::default(),
            },
        };
        RequestBuilder::with_body(
            self.client,
            operations::RefreshCampaignRecipientEstimation::path(),
            json_body(&request, None),
        )
    }

    /// `GET /api/campaign-recipient-estimation-jobs/{id}` — status of a
    /// recipient estimation job.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_campaign_recipient_estimation_job
    pub fn get_recipient_estimation_job(&self, campaign_id: &str) -> GetRecipientEstimationJob<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetCampaignRecipientEstimationJob::path(campaign_id),
        )
    }

    /// `GET /api/campaign-recipient-estimations/{id}` — a campaign's
    /// estimated recipient count.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_campaign_recipient_estimation
    pub fn get_recipient_estimation(&self, campaign_id: &str) -> GetRecipientEstimation<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetCampaignRecipientEstimation::path(campaign_id),
        )
    }
}

/// Builder for [`Campaigns::list`].
pub type ListCampaigns<'a> = RequestBuilder<
    'a,
    types::GetOmniCampaignResponseCollectionCompoundDocument,
    operations::GetCampaigns,
>;

/// Builder for [`Campaigns::get`].
pub type GetCampaign<'a> =
    RequestBuilder<'a, types::GetOmniCampaignResponseCompoundDocument, operations::GetCampaign>;

/// Builder for [`Campaigns::create`].
pub type CreateCampaign<'a> =
    RequestBuilder<'a, types::PostOmniCampaignResponse, operations::CreateCampaign>;

/// Builder for [`Campaigns::update`].
pub type UpdateCampaign<'a> =
    RequestBuilder<'a, types::PatchOmniCampaignResponse, operations::UpdateCampaign>;

/// Builder for [`Campaigns::delete`].
pub type DeleteCampaign<'a> = RequestBuilder<'a, NoContent, operations::DeleteCampaign>;

/// Builder for [`Campaigns::clone_campaign`].
pub type CloneCampaign<'a> =
    RequestBuilder<'a, types::PostOmniCampaignResponse, operations::CloneCampaign>;

/// Builder for [`Campaigns::messages`].
pub type ListCampaignMessages<'a> = RequestBuilder<
    'a,
    types::GetMessageResponseCollectionCompoundDocument,
    operations::GetMessagesForCampaign,
>;

/// Builder for [`Campaigns::get_message`].
pub type GetCampaignMessage<'a> =
    RequestBuilder<'a, types::GetMessageResponseCompoundDocument, operations::GetCampaignMessage>;

/// Builder for [`Campaigns::update_message`].
pub type UpdateCampaignMessage<'a> =
    RequestBuilder<'a, types::PatchMessageResponse, operations::UpdateCampaignMessage>;

/// Builder for [`Campaigns::variations`].
pub type ListCampaignVariations<'a> = RequestBuilder<
    'a,
    types::GetVariationResponseCollection,
    operations::GetVariationsForCampaignMessage,
>;

/// Builder for [`Campaigns::update_variation`].
pub type UpdateCampaignVariation<'a> =
    RequestBuilder<'a, types::PatchVariationResponse, operations::UpdateCampaignVariation>;

/// Builder for [`Campaigns::send`].
pub type SendCampaign<'a> =
    RequestBuilder<'a, types::PostCampaignSendJobResponse, operations::SendCampaign>;

/// Builder for [`Campaigns::get_send_job`].
pub type GetCampaignSendJob<'a> =
    RequestBuilder<'a, types::GetCampaignSendJobResponse, operations::GetCampaignSendJob>;

/// Builder for [`Campaigns::cancel_send`].
pub type CancelCampaignSend<'a> = RequestBuilder<'a, NoContent, operations::CancelCampaignSend>;

/// Builder for [`Campaigns::refresh_recipient_estimation`].
pub type RefreshRecipientEstimation<'a> = RequestBuilder<
    'a,
    types::PostCampaignRecipientEstimationJobResponse,
    operations::RefreshCampaignRecipientEstimation,
>;

/// Builder for [`Campaigns::get_recipient_estimation_job`].
pub type GetRecipientEstimationJob<'a> = RequestBuilder<
    'a,
    types::GetCampaignRecipientEstimationJobResponse,
    operations::GetCampaignRecipientEstimationJob,
>;

/// Builder for [`Campaigns::get_recipient_estimation`].
pub type GetRecipientEstimation<'a> = RequestBuilder<
    'a,
    types::GetCampaignRecipientEstimationResponse,
    operations::GetCampaignRecipientEstimation,
>;
