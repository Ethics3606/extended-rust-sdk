//! Main trading client for Extended Exchange.
//!
//! The `TradingClient` provides a unified interface to interact with the Extended
//! Exchange API, including public market data and authenticated trading operations.

use crate::api::{PrivateApi, PublicApi};
use crate::client::HttpClient;
use crate::config::EndpointConfig;
use crate::error::Result;
use crate::models::StarkAccount;
use crate::signing::StarkSigner;

/// Main trading client for Extended Exchange.
///
/// This client provides access to all API endpoints through specialized sub-modules:
/// - `public()` - Public market data (no authentication required)
/// - `private()` - Authenticated account and trading operations
///
/// # Example
///
/// ```no_run
/// use extended_rust_sdk::{TradingClient, config::testnet_config, models::StarkAccount};
///
/// #[tokio::main]
/// async fn main() -> extended_rust_sdk::error::Result<()> {
///     // Create account from Extended Exchange credentials
///     let account = StarkAccount::new(
///         "your-api-key",
///         "your-stark-public-key",
///         "your-stark-private-key",
///         "your-vault-id",
///     );
///
///     // Create trading client
///     let client = TradingClient::new(testnet_config(), account)?;
///
///     // Access public API (no auth needed)
///     let markets = client.public().get_markets().await?;
///     println!("Available markets: {}", markets.len());
///
///     // Access private API (requires auth)
///     let balance = client.private().get_balance().await?;
///     println!("Account equity: {}", balance.equity);
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct TradingClient {
    config: EndpointConfig,
    public_api: PublicApi,
    private_api: PrivateApi,
    signer: StarkSigner,
    account: StarkAccount,
}

impl TradingClient {
    /// Create a new trading client.
    ///
    /// # Arguments
    /// * `config` - Endpoint configuration (mainnet or testnet)
    /// * `account` - Stark account credentials
    ///
    /// # Returns
    /// A new `TradingClient` instance
    pub fn new(config: EndpointConfig, account: StarkAccount) -> Result<Self> {
        let public_client = HttpClient::new(config.clone())?;
        let private_client = HttpClient::with_api_key(config.clone(), &account.api_key)?;

        let signer = StarkSigner::from_hex(&account.private_key)?;

        Ok(Self {
            config,
            public_api: PublicApi::new(public_client),
            private_api: PrivateApi::new(private_client),
            signer,
            account,
        })
    }

    /// Create a public-only client (no authentication).
    ///
    /// This client can only access public market data endpoints.
    ///
    /// # Arguments
    /// * `config` - Endpoint configuration
    ///
    /// # Returns
    /// A new `PublicOnlyClient` instance
    pub fn public_only(config: EndpointConfig) -> Result<PublicOnlyClient> {
        PublicOnlyClient::new(config)
    }

    /// Get the endpoint configuration.
    pub fn config(&self) -> &EndpointConfig {
        &self.config
    }

    /// Get the Stark account credentials.
    pub fn account(&self) -> &StarkAccount {
        &self.account
    }

    /// Get the Stark signer.
    pub fn signer(&self) -> &StarkSigner {
        &self.signer
    }

    /// Access public API endpoints.
    ///
    /// Public endpoints provide market data and do not require authentication.
    pub fn public(&self) -> &PublicApi {
        &self.public_api
    }

    /// Access private API endpoints.
    ///
    /// Private endpoints require authentication and provide account/trading operations.
    pub fn private(&self) -> &PrivateApi {
        &self.private_api
    }
}

/// A client for public API access only (no authentication).
///
/// Use this when you only need to access market data without trading capabilities.
///
/// # Example
///
/// ```no_run
/// use extended_rust_sdk::{TradingClient, config::mainnet_config};
///
/// #[tokio::main]
/// async fn main() -> extended_rust_sdk::error::Result<()> {
///     let client = TradingClient::public_only(mainnet_config())?;
///
///     // Fetch market data
///     let markets = client.api().get_markets().await?;
///     let orderbook = client.api().get_orderbook("BTC-USD", Some(10)).await?;
///
///     println!("Best bid: {:?}", orderbook.best_bid());
///     println!("Best ask: {:?}", orderbook.best_ask());
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct PublicOnlyClient {
    config: EndpointConfig,
    api: PublicApi,
}

impl PublicOnlyClient {
    /// Create a new public-only client.
    pub fn new(config: EndpointConfig) -> Result<Self> {
        let http_client = HttpClient::new(config.clone())?;
        Ok(Self {
            config,
            api: PublicApi::new(http_client),
        })
    }

    /// Get the endpoint configuration.
    pub fn config(&self) -> &EndpointConfig {
        &self.config
    }

    /// Access the public API.
    pub fn api(&self) -> &PublicApi {
        &self.api
    }
}

/// A client for read-only API access (API key only, no Stark signing).
///
/// Use this when you only need to read account data without trading capabilities.
/// Supports all read endpoints: balance, positions, orders, spot balances, etc.
///
/// # Example
///
/// ```no_run
/// use extended_rust_sdk::{ReadOnlyClient, config::mainnet_config};
///
/// #[tokio::main]
/// async fn main() -> extended_rust_sdk::error::Result<()> {
///     let client = ReadOnlyClient::new(mainnet_config(), "your-api-key")?;
///
///     // Read account data (no Stark keys needed)
///     let balance = client.private().get_balance().await?;
///     let spot = client.private().get_spot_balances().await?;
///     let positions = client.private().get_positions(None).await?;
///
///     println!("True value: ${}", spot.total_notional_value());
///
///     // Also access public data
///     let markets = client.public().get_markets().await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct ReadOnlyClient {
    config: EndpointConfig,
    public_api: PublicApi,
    private_api: PrivateApi,
}

impl ReadOnlyClient {
    /// Create a new read-only client with just an API key.
    ///
    /// # Arguments
    /// * `config` - Endpoint configuration (mainnet or testnet)
    /// * `api_key` - Your Extended Exchange API key
    pub fn new(config: EndpointConfig, api_key: impl AsRef<str>) -> Result<Self> {
        let public_client = HttpClient::new(config.clone())?;
        let private_client = HttpClient::with_api_key(config.clone(), api_key.as_ref())?;

        Ok(Self {
            config,
            public_api: PublicApi::new(public_client),
            private_api: PrivateApi::new(private_client),
        })
    }

    /// Get the endpoint configuration.
    pub fn config(&self) -> &EndpointConfig {
        &self.config
    }

    /// Access public API endpoints.
    pub fn public(&self) -> &PublicApi {
        &self.public_api
    }

    /// Access private API endpoints (read-only operations).
    ///
    /// Note: Write operations (create order, cancel, withdraw) will fail
    /// as they require Stark signatures. Use `TradingClient` for trading.
    pub fn private(&self) -> &PrivateApi {
        &self.private_api
    }
}

/// Builder for creating trading clients with custom configuration.
#[derive(Debug)]
pub struct TradingClientBuilder {
    config: EndpointConfig,
    account: Option<StarkAccount>,
}

impl TradingClientBuilder {
    /// Create a new builder with the given configuration.
    pub fn new(config: EndpointConfig) -> Self {
        Self {
            config,
            account: None,
        }
    }

    /// Set the Stark account credentials.
    pub fn with_account(mut self, account: StarkAccount) -> Self {
        self.account = Some(account);
        self
    }

    /// Build a public-only client (no authentication).
    pub fn build_public(self) -> Result<PublicOnlyClient> {
        PublicOnlyClient::new(self.config)
    }

    /// Build a full trading client (requires account credentials).
    pub fn build(self) -> Result<TradingClient> {
        let account = self.account.ok_or_else(|| {
            crate::error::ExtendedError::InvalidParameter(
                "Account credentials required for trading client".to_string(),
            )
        })?;
        TradingClient::new(self.config, account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::testnet_config;

    #[test]
    fn test_public_only_client() {
        let client = TradingClient::public_only(testnet_config()).unwrap();
        assert!(!client.config().api_base_url.is_empty());
    }

    #[test]
    fn test_builder_public() {
        let client = TradingClientBuilder::new(testnet_config())
            .build_public()
            .unwrap();
        assert!(!client.config().api_base_url.is_empty());
    }
}
