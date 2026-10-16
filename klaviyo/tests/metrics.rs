//! Integration tests for the metric endpoints.
//!
//! Read-only: they page through metrics and ask for IDs that don't exist.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn page_through_metrics() -> Result {
    let client = common::client()?;
    let mut pages = client.metrics().list().pages();
    while let Some(page) = pages.next_page().await {
        page?;
    }
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn get_unknown_metric_is_rejected() -> Result {
    let client = common::client()?;
    ensure_rejected!(client.metrics().get(&unique_name!("metric")).send().await);
    Ok(())
}
