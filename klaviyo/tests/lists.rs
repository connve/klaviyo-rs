//! Integration tests for the list endpoints.
//!
//! The lifecycle test deletes the list it creates, even when a check fails.
//! Klaviyo allows 150 new lists a day per account.
//!
//! Runs against the Klaviyo test account whose private API key is in
//! `KLAVIYO_API_KEY`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

use klaviyo::types;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn list_lifecycle() -> Result {
    let client = common::client()?;
    let name = unique_name!("list");

    let request = types::ListCreateQuery {
        data: types::ListCreateQueryResourceObject {
            attributes: types::ListCreateQueryResourceObjectAttributes {
                name: name.clone(),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let list_id = client.lists().create(&request).send().await?.data.id;

    let checks: Result = async {
        let renamed = format!("{name}-renamed");
        let update = types::ListPartialUpdateQuery {
            data: types::ListPartialUpdateQueryResourceObject {
                attributes: types::ListPartialUpdateQueryResourceObjectAttributes {
                    name: Some(renamed.clone()),
                    ..Default::default()
                },
                ..Default::default()
            },
        };
        client.lists().update(&list_id, &update).send().await?;

        let fetched = client.lists().get(&list_id).send().await?;
        ensure!(
            fetched.data.attributes.name.as_deref() == Some(renamed.as_str()),
            "expected {renamed:?}, got {:?}",
            fetched.data.attributes.name
        );
        Ok(())
    }
    .await;

    let cleanup = client.lists().delete(&list_id).send().await;
    checks?;
    cleanup?;
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn page_through_lists() -> Result {
    let client = common::client()?;
    let mut pages = client.lists().list().page_size(2).pages();
    let mut fetched = 0;
    while let Some(page) = pages.next_page().await {
        page?;
        fetched += 1;
        if fetched == 3 {
            break;
        }
    }
    ensure!(fetched >= 1, "no page was fetched");
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Klaviyo test account key in KLAVIYO_API_KEY"]
async fn get_unknown_list_is_rejected() -> Result {
    let client = common::client()?;
    ensure_rejected!(client.lists().get(&unique_name!("list")).send().await);
    Ok(())
}
