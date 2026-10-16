//! Integration tests for the event endpoints.
//!
//! Read-only: they list events and ask for IDs that don't exist. No event is
//! tracked: Klaviyo processes events asynchronously, and a tracked event
//! leaves a metric behind that the API can't delete.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn list_events() -> Result {
    let client = common::client()?;
    client
        .events()
        .list()
        .sort("-datetime")
        .page_size(5)
        .send()
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn get_unknown_event_is_rejected() -> Result {
    let client = common::client()?;
    ensure_rejected!(client.events().get(&unique_name!("event")).send().await);
    Ok(())
}
