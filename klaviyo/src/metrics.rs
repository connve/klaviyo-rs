//! Metrics (event types such as `Placed Order`) and metric aggregates.
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
//! let mut pages = client.metrics().list().pages();
//! while let Some(page) = pages.next_page().await {
//!     for metric in page?.data {
//!         println!("metric {}", metric.id);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::Client;
use crate::http::json_body;
use crate::operations;
use crate::request::RequestBuilder;
use crate::types;

/// Accessor returned by [`Client::metrics`](crate::Client::metrics).
#[derive(Debug)]
pub struct Metrics<'a> {
    client: &'a Client,
}

impl<'a> Metrics<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /api/metrics` — list metrics.
    ///
    /// Filter on `integration.name`, `integration.category`. Klaviyo returns
    /// up to 200 per page and doesn't take a page size or sort order. See the
    /// [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_metrics
    pub fn list(&self) -> ListMetrics<'a> {
        RequestBuilder::new(self.client, operations::GetMetrics::path())
    }

    /// `GET /api/metrics/{id}` — get one metric.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_metric
    pub fn get(&self, metric_id: &str) -> GetMetric<'a> {
        RequestBuilder::new(self.client, operations::GetMetric::path(metric_id))
    }

    /// `POST /api/metric-aggregates` — count, sum, or unique-count a
    /// metric's events, grouped by time and dimensions.
    ///
    /// For campaign and flow performance that matches the Klaviyo UI, prefer
    /// the reporting endpoints. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/query_metric_aggregates
    pub fn query_aggregates(
        &self,
        request: &types::MetricAggregateQuery,
    ) -> QueryMetricAggregates<'a> {
        RequestBuilder::with_body(
            self.client,
            operations::QueryMetricAggregates::path(),
            json_body(request, None),
        )
    }
}

/// Builder for [`Metrics::list`].
pub type ListMetrics<'a> =
    RequestBuilder<'a, types::GetMetricResponseCollectionCompoundDocument, operations::GetMetrics>;

/// Builder for [`Metrics::get`].
pub type GetMetric<'a> =
    RequestBuilder<'a, types::GetMetricResponseCompoundDocument, operations::GetMetric>;

/// Builder for [`Metrics::query_aggregates`].
pub type QueryMetricAggregates<'a> =
    RequestBuilder<'a, types::PostMetricAggregateResponse, operations::QueryMetricAggregates>;
