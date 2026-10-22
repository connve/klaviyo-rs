//! Integration tests for the campaign endpoints.
//!
//! The draft campaign lifecycle, which needs templates and lists too, lives
//! in `tests/fan_out.rs`. Nothing here sends a campaign.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn list_campaigns() -> Result {
    let client = common::client()?;
    client
        .campaigns()
        .list()
        .filter("equals(archived,false)")
        .page_size(5)
        .send()
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn list_with_status_filter_is_rejected() -> Result {
    let client = common::client()?;
    ensure_rejected!(
        client
            .campaigns()
            .list()
            .filter("equals(status,'Draft')")
            .send()
            .await
    );
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn get_unknown_campaign_is_rejected() -> Result {
    let client = common::client()?;
    ensure_rejected!(
        client
            .campaigns()
            .get(&unique_name!("campaign"))
            .send()
            .await
    );
    Ok(())
}
