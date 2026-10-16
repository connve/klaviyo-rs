//! Integration tests for the profile endpoints.
//!
//! Profiles can't be deleted through this crate, so the write test upserts
//! one fixed profile. Concurrent runs share it, so the test only checks which
//! profile the upsert lands on.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

use klaviyo::serde_json::{Map, Value};
use klaviyo::types;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The one profile the write test upserts, shared by concurrent runs.
const TEST_PROFILE_EMAIL: &str = "klaviyo-rs-test@klaviyo-demo.com";

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn upsert_updates_the_same_profile() -> Result {
    let client = common::client()?;
    let request = types::ProfileUpsertQuery {
        data: types::ProfileUpsertQueryResourceObject {
            attributes: types::ProfileUpsertQueryResourceObjectAttributes {
                email: Some(TEST_PROFILE_EMAIL.to_owned()),
                properties: Some(Map::from_iter([(
                    "klaviyo_rs_last_run".to_owned(),
                    Value::from(unique_name!("run")),
                )])),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let first = client.profiles().upsert(&request).send().await?;
    let second = client.profiles().upsert(&request).send().await?;
    let id = first.data.id.ok_or("the upsert returned no profile ID")?;
    ensure!(
        second.data.id.as_deref() == Some(id.as_str()),
        "upserts landed on {id} and {:?}",
        second.data.id
    );

    let fetched = client.profiles().get(&id).send().await?;
    ensure!(
        fetched.data.attributes.email.as_deref() == Some(TEST_PROFILE_EMAIL),
        "fetched {:?}",
        fetched.data.attributes.email
    );
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn list_profiles_with_subscriptions() -> Result {
    let client = common::client()?;
    client
        .profiles()
        .list()
        .additional_fields("profile", &["subscriptions"])
        .page_size(5)
        .send()
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn get_unknown_profile_is_rejected() -> Result {
    let client = common::client()?;
    ensure_rejected!(client.profiles().get(&unique_name!("profile")).send().await);
    Ok(())
}
