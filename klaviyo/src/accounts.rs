//! The account an API key belongs to.
//!
//! ```no_run
//! use klaviyo::Client;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .api_key(std::env::var("KLAVIYO_API_KEY")?)
//!     .build()?;
//!
//! let accounts = client.accounts().list().send().await?;
//! for account in &accounts.data {
//!     println!("API key belongs to account {}", account.id);
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::Client;
use crate::operations;
use crate::request::RequestBuilder;
use crate::types;

/// Accessor returned by [`Client::accounts`](crate::Client::accounts).
#[derive(Debug)]
pub struct Accounts<'a> {
    client: &'a Client,
}

impl<'a> Accounts<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /api/accounts` — the account the API key belongs to, as a
    /// one-element collection.
    ///
    /// Rate-limited to 1/s and 15/min. See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_accounts
    pub fn list(&self) -> ListAccounts<'a> {
        RequestBuilder::new(self.client, operations::GetAccounts::path())
    }

    /// `GET /api/accounts/{id}` — get the account by ID. Only the key's own
    /// account can be read.
    ///
    /// See the [Klaviyo docs][docs].
    ///
    /// [docs]: https://developers.klaviyo.com/en/reference/get_account
    pub fn get(&self, account_id: &str) -> GetAccount<'a> {
        RequestBuilder::new(self.client, operations::GetAccount::path(account_id))
    }
}

/// Builder for [`Accounts::list`].
pub type ListAccounts<'a> =
    RequestBuilder<'a, types::GetAccountResponseCollection, operations::GetAccounts>;

/// Builder for [`Accounts::get`].
pub type GetAccount<'a> = RequestBuilder<'a, types::GetAccountResponse, operations::GetAccount>;
