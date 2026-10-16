//! Integration tests for the reporting endpoints.
//!
//! Reports are limited to 2 a minute and 225 a day per account, so this
//! file runs one report.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

use klaviyo::types;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn campaign_values_for_the_last_30_days() -> Result {
    let client = common::client()?;
    let metrics = client.metrics().list().send().await?;
    let Some(metric) = metrics.data.first() else {
        eprintln!("Skipping: the test account has no metric to use as the conversion metric.");
        return Ok(());
    };

    let query = types::CampaignValuesRequestDto {
        data: types::CampaignValuesRequestDtoResourceObject {
            attributes: types::CampaignValuesRequestDtoResourceObjectAttributes {
                conversion_metric_id: metric.id.clone(),
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
    };

    let mut pages = client.reporting().campaign_values(&query).pages();
    let mut fetched = 0;
    while let Some(page) = pages.next_page().await {
        page?;
        fetched += 1;
    }
    ensure!(fetched >= 1, "the report returned no page");
    Ok(())
}
