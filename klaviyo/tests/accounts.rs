//! Integration tests for the account endpoints.
//!
//! Read-only. Accounts are rate-limited to 1/s and 15/min, so this file
//! makes two calls in total.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn the_key_reads_its_own_account() -> Result {
    let client = common::client()?;
    let accounts = client.accounts().list().send().await?;
    ensure!(
        accounts.data.len() == 1,
        "expected the key's own account only, got {}",
        accounts.data.len()
    );

    let id = &accounts.data[0].id;
    let account = client.accounts().get(id).send().await?;
    ensure!(
        account.data.id == *id,
        "listed {id}, fetched {}",
        account.data.id
    );
    Ok(())
}
