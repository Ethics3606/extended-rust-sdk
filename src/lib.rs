//! # Extended Rust SDK
//!
//! A Rust SDK for the Extended crypto exchange (Starknet-based perpetual trading).
//!
//! ## Overview
//!
//! Extended is a hybrid CLOB exchange where order processing, matching, and risk
//! assessment happen off-chain, while settlement occurs on-chain via Starknet.
//!
//! This SDK provides:
//! - **Public API**: Market data, orderbooks, trades, candles (no authentication required)
//! - **Private API**: Account management, order placement, positions, balances
//! - **Stark Signing**: EIP-712 based key derivation and order signing
//!
//! ## Quick Start
//!
//! ### Public Data (No Authentication)
//!
//! ```no_run
//! use extended_rust_sdk::{TradingClient, config::testnet_config};
//!
//! #[tokio::main]
//! async fn main() -> extended_rust_sdk::error::Result<()> {
//!     // Create a public-only client
//!     let client = TradingClient::public_only(testnet_config())?;
//!
//!     // Fetch available markets
//!     let markets = client.api().get_markets().await?;
//!     println!("Found {} markets", markets.len());
//!
//!     // Get orderbook for BTC-USD
//!     let orderbook = client.api().get_orderbook("BTC-USD", Some(10)).await?;
//!     if let (Some(bid), Some(ask)) = (orderbook.best_bid(), orderbook.best_ask()) {
//!         println!("BTC-USD: {} / {}", bid.price, ask.price);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Read-Only Access (API Key Only)
//!
//! For apps that only need to read account data (no trading):
//!
//! ```no_run
//! use extended_rust_sdk::{ReadOnlyClient, config::mainnet_config};
//!
//! #[tokio::main]
//! async fn main() -> extended_rust_sdk::error::Result<()> {
//!     // Only needs API key - no Stark keys required
//!     let client = ReadOnlyClient::new(mainnet_config(), "your-api-key")?;
//!
//!     // Read account data
//!     let balance = client.private().get_balance().await?;
//!     let spot = client.private().get_spot_balances().await?;
//!
//!     println!("Equity: {}", balance.equity);
//!     println!("True value: ${}", spot.total_notional_value());
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Full Trading Access (API Key + Stark Keys)
//!
//! ```no_run
//! use extended_rust_sdk::{TradingClient, config::testnet_config, models::StarkAccount};
//!
//! #[tokio::main]
//! async fn main() -> extended_rust_sdk::error::Result<()> {
//!     // Create account from Extended Exchange credentials
//!     let account = StarkAccount::new(
//!         "your-api-key",
//!         "your-stark-public-key",
//!         "your-stark-private-key",
//!         "your-vault-id",
//!     );
//!
//!     let client = TradingClient::new(testnet_config(), account)?;
//!
//!     // Get account balance
//!     let balance = client.private().get_balance().await?;
//!     println!("Equity: {}, Available: {}", balance.equity, balance.available_for_trade);
//!
//!     // Get open positions
//!     let positions = client.private().get_positions(None).await?;
//!     for pos in positions {
//!         println!("{}: {} @ {} (PnL: {})",
//!             pos.market, pos.size, pos.entry_price, pos.unrealized_pnl);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! Use `mainnet_config()` for production or `testnet_config()` for Sepolia testnet:
//!
//! ```
//! use extended_rust_sdk::config::{mainnet_config, testnet_config};
//!
//! let mainnet = mainnet_config();
//! let testnet = testnet_config();
//! ```
//!
//! ## Order Signing
//!
//! Orders must be signed with your Stark private key before submission:
//!
//! ```no_run
//! use rust_decimal_macros::dec;
//! use extended_rust_sdk::{
//!     models::{OrderBuilder, OrderSide},
//!     signing::{StarkSigner, sign_order},
//! };
//!
//! // Create signer from your Stark private key
//! let signer = StarkSigner::from_hex("0x...your-stark-private-key...")?;
//!
//! // Build the order
//! let order = OrderBuilder::limit("BTC-USD", OrderSide::Buy, dec!(50000), dec!(0.01))
//!     .post_only(true)
//!     .build(dec!(0.0001), 1); // fee, nonce
//!
//! // Sign the order
//! let signed_order = sign_order(order, &signer)?;
//!
//! // Now submit via client.private().create_order(signed_order)
//! # Ok::<(), extended_rust_sdk::error::ExtendedError>(())
//! ```
//!
//! ## Dependency Version Policy
//!
//! This SDK uses recent/current versions of all dependencies to ensure compatibility
//! with other crates in your project. When updating dependencies, always use the
//! latest stable versions. Key dependencies include:
//!
//! - `alloy` >= 1.4.0 - For EIP-712 signing and Ethereum primitives
//! - `reqwest` >= 0.13.0 - HTTP client
//! - `tokio` >= 1.49.0 - Async runtime
//! - `serde` >= 1.0.228 - Serialization
//!
//! Run `cargo update` regularly to keep dependencies current.

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod signing;
mod trading_client;

// Re-export main types at crate root
pub use trading_client::{PublicOnlyClient, ReadOnlyClient, TradingClient, TradingClientBuilder};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::api::{PrivateApi, PublicApi};
    pub use crate::config::{mainnet_config, testnet_config, EndpointConfig};
    pub use crate::error::{ExtendedError, Result};
    pub use crate::models::*;
    pub use crate::signing::{StarkSigner, sign_order};
    pub use crate::{PublicOnlyClient, ReadOnlyClient, TradingClient, TradingClientBuilder};
}
