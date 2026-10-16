//! Integration tests for the flow endpoints.
//!
//! Read-only: they list flows and ask for IDs that don't exist. No flow
//! status is changed.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn list_flows() -> Result {
    let client = common::client()?;
    client.flows().list().page_size(5).send().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn get_unknown_flow_is_rejected() -> Result {
    let client = common::client()?;
    ensure_rejected!(client.flows().get(&unique_name!("flow")).send().await);
    Ok(())
}
