//! Flows and flow status.
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
//! let live = client
//!     .flows()
//!     .list()
//!     .filter("equals(status,'live')")
//!     .send()
//!     .await?;
//! for flow in &live.data {
//!     println!("live flow {}", flow.id);
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::Client;
use crate::http::json_body;
use crate::operations;
use crate::request::RequestBuilder;
use crate::types;

/// Accessor returned by [`Client::flows`](crate::Client::flows).
#[derive(Debug)]
pub struct Flows<'a> {
    client: &'a Client,
}

impl<'a> Flows<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /api/flows` — list flows.
    ///
    /// Filter on `id`, `name`, `status`, `archived`, `created`, `updated`,
    /// `trigger_type`. Up to 50 per page. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_flows
    pub fn list(&self) -> ListFlows<'a> {
        RequestBuilder::new(self.client, operations::GetFlows::path())
    }

    /// `GET /api/flows/{id}` — get one flow.
    ///
    /// `additional_fields("flow", &["definition"])` adds the full flow
    /// definition. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_flow
    pub fn get(&self, flow_id: &str) -> GetFlow<'a> {
        RequestBuilder::new(self.client, operations::GetFlow::path(flow_id))
    }

    /// `PATCH /api/flows/{id}` — set a flow and all its actions to `draft`,
    /// `manual`, or `live`.
    ///
    /// `data.id` in the body is set to `flow_id`. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/update_flow
    pub fn update(&self, flow_id: &str, request: &types::FlowUpdateQuery) -> UpdateFlow<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::UpdateFlow::path(flow_id),
            json_body(request, Some(flow_id)),
        )
    }
}

/// Builder for [`Flows::list`].
pub type ListFlows<'a> =
    RequestBuilder<'a, types::GetFlowResponseCollectionCompoundDocument, operations::GetFlows>;

/// Builder for [`Flows::get`].
pub type GetFlow<'a> =
    RequestBuilder<'a, types::GetFlowV2ResponseCompoundDocument, operations::GetFlow>;

/// Builder for [`Flows::update`].
pub type UpdateFlow<'a> = RequestBuilder<'a, types::PatchFlowResponse, operations::UpdateFlow>;
