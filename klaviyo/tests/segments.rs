//! Integration tests for the segment endpoints.
//!
//! Read-only: they list segments and ask for IDs that don't exist.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn list_segments() -> Result {
    let client = common::client()?;
    client.segments().list().send().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn get_unknown_segment_is_rejected() -> Result {
    let client = common::client()?;
    ensure_rejected!(client.segments().get(&unique_name!("segment")).send().await);
    Ok(())
}
