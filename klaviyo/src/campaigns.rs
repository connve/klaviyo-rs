//! Campaigns, campaign messages, template assignment, send jobs, and
//! recipient estimates.
//!
//! A campaign is created as a draft with its messages; assigning a template
//! copies it into a message. Sends can't be undone.
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
//! let drafts = client
//!     .campaigns()
//!     .list("and(equals(messages.channel,'email'),equals(status,'Draft'))")
//!     .sort("-created_at")
//!     .send()
//!     .await?;
//!
//! for campaign in &drafts.data {
//!     println!("draft campaign {}", campaign.id);
//! }
//!
//! // Copy a template into the first message of a campaign.
//! let campaign_id = "01JABCDEF";
//! let messages = client.campaigns().messages(campaign_id).send().await?;
//! if let Some(message) = messages.data.first() {
//!     client
//!         .campaigns()
//!         .assign_template(&message.id, "RipRmi")
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
    /// Klaviyo requires a channel in `filter`, e.g.
    /// `equals(messages.channel,'email')`; add conditions with `and(...)`.
    /// Up to 100 per page. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_campaigns
    pub fn list(&self, filter: &str) -> ListCampaigns<'a> {
        RequestBuilder::new(self.client, operations::GetCampaigns::path()).query("filter", filter)
    }

    /// `GET /api/campaigns/{id}` — get one campaign.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_campaign
    pub fn get(&self, campaign_id: &str) -> GetCampaign<'a> {
        RequestBuilder::new(self.client, operations::GetCampaign::path(campaign_id))
    }

    /// `POST /api/campaigns` — create a draft campaign with its messages.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/create_campaign
    pub fn create(&self, request: &types::CampaignCreateQuery) -> CreateCampaign<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CreateCampaign::path(),
            json_body(request, None),
        )
    }

    /// `PATCH /api/campaigns/{id}` — update a campaign's name, audiences,
    /// send strategy, or options.
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
    /// [docs]: https://developers.klaviyo.com/en/reference/create_campaign_clone
    pub fn clone_campaign(
        &self,
        campaign_id: &str,
        request: &types::CampaignCloneQuery,
    ) -> CloneCampaign<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::CreateCampaignClone::path(),
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

    /// `PATCH /api/campaign-messages/{id}` — update a message's label and
    /// content, such as the subject line, preview text, and sender.
    ///
    /// `data.id` in the body is set to `message_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/update_campaign_message
    pub fn update_message(
        &self,
        message_id: &str,
        request: &types::CampaignMessagePartialUpdateQuery,
    ) -> UpdateCampaignMessage<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::UpdateCampaignMessage::path(message_id),
            json_body(request, Some(message_id)),
        )
    }

    /// `POST /api/campaign-message-assign-template` — copy a template into a
    /// campaign message.
    ///
    /// Later edits to the template don't reach the message. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/assign_template_to_campaign_message
    pub fn assign_template(&self, message_id: &str, template_id: &str) -> AssignTemplate<'a> {
        let request = types::CampaignMessageAssignTemplateQuery {
            data: types::CampaignMessageAssignTemplateQueryResourceObject {
                id: message_id.to_owned(),
                relationships: types::CampaignMessageAssignTemplateQueryResourceObjectRelationships {
                    template:
                        types::CampaignMessageAssignTemplateQueryResourceObjectRelationshipsTemplate {
                            data: Some(
                                types::CampaignMessageAssignTemplateQueryResourceObjectRelationshipsTemplateData {
                                    id: template_id.to_owned(),
                                    type_: Default::default(),
                                },
                            ),
                        },
                },
                type_: Default::default(),
            },
        };
        RequestBuilder::with_body(
            self.client,
            operations::AssignTemplateToCampaignMessage::path(),
            json_body(&request, None),
        )
    }

    /// `GET /api/campaign-messages/{id}/template` — the template assigned to
    /// a campaign message.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_template_for_campaign_message
    pub fn message_template(&self, message_id: &str) -> GetCampaignMessageTemplate<'a> {
        RequestBuilder::new(
            self.client,
            operations::GetTemplateForCampaignMessage::path(message_id),
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
    types::GetCampaignResponseCollectionCompoundDocument,
    operations::GetCampaigns,
>;

/// Builder for [`Campaigns::get`].
pub type GetCampaign<'a> =
    RequestBuilder<'a, types::GetCampaignResponseCompoundDocument, operations::GetCampaign>;

/// Builder for [`Campaigns::create`].
pub type CreateCampaign<'a> =
    RequestBuilder<'a, types::PostCampaignResponse, operations::CreateCampaign>;

/// Builder for [`Campaigns::update`].
pub type UpdateCampaign<'a> =
    RequestBuilder<'a, types::PatchCampaignResponse, operations::UpdateCampaign>;

/// Builder for [`Campaigns::delete`].
pub type DeleteCampaign<'a> = RequestBuilder<'a, NoContent, operations::DeleteCampaign>;

/// Builder for [`Campaigns::clone_campaign`].
pub type CloneCampaign<'a> =
    RequestBuilder<'a, types::PostCampaignResponse, operations::CreateCampaignClone>;

/// Builder for [`Campaigns::messages`].
pub type ListCampaignMessages<'a> = RequestBuilder<
    'a,
    types::GetCampaignMessageResponseCollectionCompoundDocument,
    operations::GetMessagesForCampaign,
>;

/// Builder for [`Campaigns::get_message`].
pub type GetCampaignMessage<'a> = RequestBuilder<
    'a,
    types::GetCampaignMessageResponseCompoundDocument,
    operations::GetCampaignMessage,
>;

/// Builder for [`Campaigns::update_message`].
pub type UpdateCampaignMessage<'a> =
    RequestBuilder<'a, types::PatchCampaignMessageResponse, operations::UpdateCampaignMessage>;

/// Builder for [`Campaigns::assign_template`].
pub type AssignTemplate<'a> = RequestBuilder<
    'a,
    types::PostCampaignMessageResponse,
    operations::AssignTemplateToCampaignMessage,
>;

/// Builder for [`Campaigns::message_template`].
pub type GetCampaignMessageTemplate<'a> =
    RequestBuilder<'a, types::GetTemplateResponse, operations::GetTemplateForCampaignMessage>;

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
