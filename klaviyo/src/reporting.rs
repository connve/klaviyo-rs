//! Campaign and flow reports, matching the numbers in the Klaviyo UI.
//!
//! Reports are limited to 2 a minute and 225 a day per account, so query
//! everything you need at once and page through the result.
//!
//! ```no_run
//! use klaviyo::{types, Client};
//!
//! # async fn run(client: Client, query: types::CampaignValuesRequestDto) -> klaviyo::Result<()> {
//! let mut pages = client.reporting().campaign_values(&query).pages();
//! while let Some(page) = pages.next_page().await {
//!     let report = page?;
//!     println!("{} rows", report.data.attributes.results.len());
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::Client;
use crate::http::json_body;
use crate::operations;
use crate::request::RequestBuilder;
use crate::types;

/// Accessor returned by [`Client::reporting`](crate::Client::reporting).
#[derive(Debug)]
pub struct Reporting<'a> {
    client: &'a Client,
}

impl<'a> Reporting<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /api/campaign-values-reports` — statistics per campaign over a
    /// timeframe, such as opens, clicks, and conversion value.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/query_campaign_values
    pub fn campaign_values(
        &self,
        request: &types::CampaignValuesRequestDto,
    ) -> QueryCampaignValues<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::QueryCampaignValues::path(),
            json_body(request, None),
        )
    }

    /// `POST /api/flow-values-reports` — statistics per flow or flow message
    /// over a timeframe.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/query_flow_values
    pub fn flow_values(&self, request: &types::FlowValuesRequestDto) -> QueryFlowValues<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::QueryFlowValues::path(),
            json_body(request, None),
        )
    }

    /// `POST /api/flow-series-reports` — flow statistics as a time series.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/query_flow_series
    pub fn flow_series(&self, request: &types::FlowSeriesRequestDto) -> QueryFlowSeries<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::QueryFlowSeries::path(),
            json_body(request, None),
        )
    }
}

/// Builder for [`Reporting::campaign_values`].
pub type QueryCampaignValues<'a> =
    RequestBuilder<'a, types::PostCampaignValuesResponseDto, operations::QueryCampaignValues>;

/// Builder for [`Reporting::flow_values`].
pub type QueryFlowValues<'a> =
    RequestBuilder<'a, types::PostFlowValuesResponseDto, operations::QueryFlowValues>;

/// Builder for [`Reporting::flow_series`].
pub type QueryFlowSeries<'a> =
    RequestBuilder<'a, types::PostFlowSeriesResponseDto, operations::QueryFlowSeries>;
