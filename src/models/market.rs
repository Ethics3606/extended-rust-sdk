//! Market-related models.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use super::PriceQuantity;

/// Helper to deserialize string numbers as Decimal.
fn decimal_from_string<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<Decimal>().map_err(serde::de::Error::custom)
}

/// Helper to deserialize optional string numbers as Option<Decimal>.
fn option_decimal_from_string<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => s.parse::<Decimal>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// Helper to deserialize string numbers as i32.
fn i32_from_string<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<i32>().map_err(serde::de::Error::custom)
}

/// L2 (Starknet) configuration for a market.
/// Contains asset IDs and resolutions needed for order signing.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L2Config {
    /// L2 type (e.g., "STARKNET").
    #[serde(rename = "type")]
    pub l2_type: String,
    /// Collateral asset ID (hex string, e.g., "0x1" for USDC).
    pub collateral_id: String,
    /// Collateral asset resolution (10^decimals, e.g., 1000000 for 6 decimals).
    pub collateral_resolution: i64,
    /// Synthetic asset ID (hex string, e.g., "0x2" for BTC).
    pub synthetic_id: String,
    /// Synthetic asset resolution (10^decimals).
    pub synthetic_resolution: i64,
}

/// Market information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    /// Market identifier (e.g., "BTC-USD").
    pub name: String,
    /// UI display name.
    #[serde(default)]
    pub ui_name: Option<String>,
    /// Market category (e.g., "DeFi", "Layer1").
    #[serde(default)]
    pub category: Option<String>,
    /// Base asset name (e.g., "BTC").
    pub asset_name: String,
    /// Base asset precision (decimal places).
    pub asset_precision: u32,
    /// Quote/collateral asset name (e.g., "USD").
    pub collateral_asset_name: String,
    /// Collateral asset precision (decimal places).
    pub collateral_asset_precision: u32,
    /// Whether the market is active.
    pub active: bool,
    /// Market status.
    pub status: MarketStatus,
    /// Trading configuration.
    pub trading_config: MarketConfig,
    /// Current market statistics.
    pub market_stats: MarketStats,
    /// L2 (Starknet) configuration for signing.
    pub l2_config: L2Config,
}

impl Market {
    /// Get base asset (alias for asset_name).
    pub fn base_asset(&self) -> &str {
        &self.asset_name
    }

    /// Get quote asset (alias for collateral_asset_name).
    pub fn quote_asset(&self) -> &str {
        &self.collateral_asset_name
    }

    /// Get the trading config (alias for trading_config).
    pub fn config(&self) -> &MarketConfig {
        &self.trading_config
    }

    /// Get the market stats (alias for market_stats).
    pub fn stats(&self) -> &MarketStats {
        &self.market_stats
    }

    /// Get the L2 (Starknet) configuration.
    pub fn l2_config(&self) -> &L2Config {
        &self.l2_config
    }

    /// Get synthetic asset ID for signing (hex string).
    pub fn synthetic_asset_id(&self) -> &str {
        &self.l2_config.synthetic_id
    }

    /// Get synthetic asset resolution for signing.
    pub fn synthetic_resolution(&self) -> i64 {
        self.l2_config.synthetic_resolution
    }

    /// Get collateral asset ID for signing (hex string).
    pub fn collateral_asset_id(&self) -> &str {
        &self.l2_config.collateral_id
    }

    /// Get collateral asset resolution for signing.
    pub fn collateral_resolution(&self) -> i64 {
        self.l2_config.collateral_resolution
    }
}

/// Market status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketStatus {
    /// Market is active, all order types permitted.
    Active,
    /// Market is in reduce-only mode, only reduce-only orders allowed.
    ReduceOnly,
    /// Market is delisted, trading no longer permitted.
    Delisted,
    /// Market is in prelisting stage, trading not yet available.
    Prelisted,
    /// Market is completely disabled, trading not allowed.
    Disabled,
}

/// Risk factor tier for position-based leverage limits.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskFactorConfig {
    /// Upper bound of position value for this tier.
    #[serde(deserialize_with = "decimal_from_string")]
    pub upper_bound: Decimal,
    /// Risk factor (1/max_leverage) for this tier.
    #[serde(deserialize_with = "decimal_from_string")]
    pub risk_factor: Decimal,
}

/// Market configuration parameters.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketConfig {
    /// Minimum order size.
    #[serde(deserialize_with = "decimal_from_string")]
    pub min_order_size: Decimal,
    /// Minimum quantity increment (step size).
    #[serde(deserialize_with = "decimal_from_string")]
    pub min_order_size_change: Decimal,
    /// Minimum price increment (tick size).
    #[serde(deserialize_with = "decimal_from_string")]
    pub min_price_change: Decimal,
    /// Maximum market order value.
    #[serde(deserialize_with = "decimal_from_string")]
    pub max_market_order_value: Decimal,
    /// Maximum limit order value.
    #[serde(deserialize_with = "decimal_from_string")]
    pub max_limit_order_value: Decimal,
    /// Maximum position value.
    #[serde(deserialize_with = "decimal_from_string")]
    pub max_position_value: Decimal,
    /// Maximum leverage allowed.
    #[serde(deserialize_with = "decimal_from_string")]
    pub max_leverage: Decimal,
    /// Maximum number of open orders.
    #[serde(deserialize_with = "i32_from_string")]
    pub max_num_orders: i32,
    /// Limit price cap (max price for buys as fraction above mark).
    #[serde(deserialize_with = "decimal_from_string")]
    pub limit_price_cap: Decimal,
    /// Limit price floor (min price for sells as fraction below mark).
    #[serde(deserialize_with = "decimal_from_string")]
    pub limit_price_floor: Decimal,
    /// Risk factor configuration tiers.
    #[serde(default)]
    pub risk_factor_config: Vec<RiskFactorConfig>,
}

impl MarketConfig {
    /// Alias for min_price_change (tick size).
    pub fn tick_size(&self) -> Decimal {
        self.min_price_change
    }

    /// Alias for min_order_size_change (step size).
    pub fn step_size(&self) -> Decimal {
        self.min_order_size_change
    }

    /// Round a price down to the market's tick size.
    pub fn round_price_down(&self, price: Decimal) -> Decimal {
        (price / self.min_price_change).floor() * self.min_price_change
    }

    /// Round a price up to the market's tick size.
    pub fn round_price_up(&self, price: Decimal) -> Decimal {
        (price / self.min_price_change).ceil() * self.min_price_change
    }

    /// Round a quantity down to the market's step size.
    pub fn round_qty_down(&self, quantity: Decimal) -> Decimal {
        (quantity / self.min_order_size_change).floor() * self.min_order_size_change
    }

    /// Round a quantity up to the market's step size.
    pub fn round_qty_up(&self, quantity: Decimal) -> Decimal {
        (quantity / self.min_order_size_change).ceil() * self.min_order_size_change
    }

    /// Get the number of decimal places for prices.
    pub fn price_precision(&self) -> u32 {
        self.min_price_change.scale()
    }

    /// Get the number of decimal places for quantities.
    pub fn qty_precision(&self) -> u32 {
        self.min_order_size_change.scale()
    }
}

/// Market trading statistics.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketStats {
    /// Market name (only present when fetched directly via get_market_stats).
    #[serde(default)]
    pub market: Option<String>,
    /// Current mark price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub mark_price: Decimal,
    /// Current index price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub index_price: Decimal,
    /// Last traded price.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub last_price: Option<Decimal>,
    /// Best ask price.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub ask_price: Option<Decimal>,
    /// Best bid price.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub bid_price: Option<Decimal>,
    /// 24h high price.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub daily_high: Option<Decimal>,
    /// 24h low price.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub daily_low: Option<Decimal>,
    /// 24h trading volume in quote asset.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub daily_volume: Option<Decimal>,
    /// 24h trading volume in base asset.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub daily_volume_base: Option<Decimal>,
    /// 24h price change.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub daily_price_change: Option<Decimal>,
    /// 24h price change percentage.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub daily_price_change_percentage: Option<Decimal>,
    /// Open interest in quote asset.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub open_interest: Option<Decimal>,
    /// Open interest in base asset.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub open_interest_base: Option<Decimal>,
    /// Current funding rate (hourly).
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub funding_rate: Option<Decimal>,
    /// Next funding rate timestamp (Unix ms).
    #[serde(default)]
    pub next_funding_rate: Option<i64>,
}

/// Order book snapshot.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    /// Market name.
    pub market: String,
    /// Bid orders (buy side), sorted by price descending.
    pub bids: Vec<PriceQuantity>,
    /// Ask orders (sell side), sorted by price ascending.
    pub asks: Vec<PriceQuantity>,
    /// Timestamp of the snapshot (Unix ms).
    pub timestamp: i64,
    /// Sequence number for updates.
    pub sequence: Option<i64>,
}

impl OrderBook {
    /// Get the best bid price.
    pub fn best_bid(&self) -> Option<&PriceQuantity> {
        self.bids.first()
    }

    /// Get the best ask price.
    pub fn best_ask(&self) -> Option<&PriceQuantity> {
        self.asks.first()
    }

    /// Get the mid price.
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid.price + ask.price) / Decimal::from(2)),
            _ => None,
        }
    }

    /// Get the spread.
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.price - bid.price),
            _ => None,
        }
    }
}

/// Funding rate information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingRate {
    /// Market name.
    pub market: String,
    /// Funding rate (hourly).
    #[serde(deserialize_with = "decimal_from_string")]
    pub funding_rate: Decimal,
    /// Funding time (Unix timestamp ms).
    pub funding_time: i64,
}

/// Open interest data point.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenInterest {
    /// Market name.
    pub market: String,
    /// Open interest value.
    #[serde(deserialize_with = "decimal_from_string")]
    pub open_interest: Decimal,
    /// Timestamp (Unix ms).
    pub timestamp: i64,
}

/// Parameters for fetching markets.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMarketsParams {
    /// Filter by market status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MarketStatus>,
}
