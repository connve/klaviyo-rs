//! Offline tests for the reporting endpoints, including report pagination.
//!
//! Runs against a local `wiremock` server and needs nothing else, so it
//! isn't `#[ignore]`d.

use klaviyo::{types, Client, Error};
use serde_json::{json, Value};
use wiremock::matchers::{body_partial_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn client(server: &MockServer) -> klaviyo::Result<Client> {
    Client::builder()
        .api_key("pk_test")
        .base_url(server.uri())
        .build()
}

fn request() -> types::CampaignValuesRequestDto {
    types::CampaignValuesRequestDto {
        data: types::CampaignValuesRequestDtoResourceObject {
            attributes: types::CampaignValuesRequestDtoResourceObjectAttributes {
                conversion_metric_id: "RESQ6t".to_owned(),
                statistics: vec![
                    types::CampaignValuesRequestDtoResourceObjectAttributesStatisticsItem::Opens,
                    types::CampaignValuesRequestDtoResourceObjectAttributesStatisticsItem::Clicks,
                ],
                timeframe: types::Timeframe {
                    key: types::TimeframeKey::Last30Days,
                }
                .into(),
                filter: None,
                group_by: None,
            },
            type_: Default::default(),
        },
    }
}

/// The body `request()` must serialize to.
fn query() -> Value {
    json!({
        "data": {
            "type": "campaign-values-report",
            "attributes": {
                "statistics": ["opens", "clicks"],
                "timeframe": { "key": "last_30_days" },
                "conversion_metric_id": "RESQ6t"
            }
        }
    })
}

fn report_page(next: Option<&str>) -> Value {
    json!({
        "data": { "type": "campaign-values-report", "attributes": { "results": [] } },
        "links": { "self": "https://a.klaviyo.com/api/campaign-values-reports", "next": next }
    })
}

#[tokio::test]
async fn campaign_values_pages_resend_the_query_with_page_cursor() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/campaign-values-reports"))
        .and(query_param_is_missing("page_cursor"))
        .and(body_partial_json(query()))
        .respond_with(ResponseTemplate::new(200).set_body_json(report_page(Some(
            "https://a.klaviyo.com/api/campaign-values-reports?page_cursor=second",
        ))))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/campaign-values-reports"))
        .and(query_param("page_cursor", "second"))
        .and(body_partial_json(query()))
        .respond_with(ResponseTemplate::new(200).set_body_json(report_page(None)))
        .expect(1)
        .mount(&server)
        .await;

    let request = request();
    let client = client(&server)?;
    let mut pages = client.reporting().campaign_values(&request).pages();
    let mut fetched = 0;
    while let Some(page) = pages.next_page().await {
        page?;
        fetched += 1;
    }
    assert_eq!(fetched, 2);
    Ok(())
}

#[tokio::test]
async fn a_next_link_without_a_cursor_is_reported_after_its_page() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/campaign-values-reports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(report_page(Some(
            "https://a.klaviyo.com/api/campaign-values-reports?offset=50",
        ))))
        .expect(1)
        .mount(&server)
        .await;

    let request = request();
    let client = client(&server)?;
    let mut pages = client.reporting().campaign_values(&request).pages();

    pages.next_page().await.expect("a first page")?;
    let next = pages
        .next_page()
        .await
        .expect("the broken link is reported");
    assert!(
        matches!(next, Err(Error::InvalidNextPage { .. })),
        "{next:?}"
    );
    assert!(pages.next_page().await.is_none());
    Ok(())
}
