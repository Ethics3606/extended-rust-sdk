//! Trade-related models.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use super::OrderSide;

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

/// Public trade (from trade feed).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicTrade {
    /// Trade ID.
    pub id: String,
    /// Market name.
    pub market: String,
    /// Trade price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub price: Decimal,
    /// Trade quantity.
    #[serde(deserialize_with = "decimal_from_string")]
    pub quantity: Decimal,
    /// Trade side (taker side).
    pub side: OrderSide,
    /// Trade timestamp (Unix ms).
    pub timestamp: i64,
}

/// User's trade (fill).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Trade ID.
    pub id: String,
    /// Order ID.
    #[serde(default)]
    pub order_id: Option<String>,
    /// Market name.
    pub market: String,
    /// Trade side.
    pub side: OrderSide,
    /// Trade price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub price: Decimal,
    /// Trade quantity.
    #[serde(deserialize_with = "decimal_from_string")]
    pub quantity: Decimal,
    /// Trade fee.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub fee: Option<Decimal>,
    /// Fee asset (usually quote asset).
    #[serde(default)]
    pub fee_asset: Option<String>,
    /// Whether this was a maker trade.
    #[serde(default)]
    pub is_maker: Option<bool>,
    /// Realized PnL from this trade.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub realized_pnl: Option<Decimal>,
    /// Trade timestamp (Unix ms).
    pub timestamp: i64,
}

impl Trade {
    /// Get the trade value (price * quantity).
    pub fn value(&self) -> Decimal {
        self.price * self.quantity
    }

    /// Get the fee, defaulting to zero if not present.
    pub fn get_fee(&self) -> Decimal {
        self.fee.unwrap_or(Decimal::ZERO)
    }

    /// Get the net value (value - fee for buy, value + fee for sell).
    pub fn net_value(&self) -> Decimal {
        let fee = self.get_fee();
        match self.side {
            OrderSide::Buy => self.value() + fee,
            OrderSide::Sell => self.value() - fee,
        }
    }
}

/// Funding payment.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingPayment {
    /// Market name.
    pub market: String,
    /// Funding rate applied.
    #[serde(deserialize_with = "decimal_from_string")]
    pub funding_rate: Decimal,
    /// Position size at funding time.
    #[serde(deserialize_with = "decimal_from_string")]
    pub position_size: Decimal,
    /// Payment amount (positive = received, negative = paid).
    #[serde(deserialize_with = "decimal_from_string")]
    pub payment: Decimal,
    /// Funding timestamp.
    pub timestamp: i64,
}

impl FundingPayment {
    /// Check if funding was received (positive).
    pub fn is_received(&self) -> bool {
        self.payment.is_sign_positive()
    }

    /// Check if funding was paid (negative).
    pub fn is_paid(&self) -> bool {
        self.payment.is_sign_negative()
    }
}

/// Parameters for fetching trades.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTradesParams {
    /// Filter by market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    /// Filter by order ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// Start timestamp (Unix ms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// End timestamp (Unix ms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<i64>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Parameters for fetching public trades.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublicTradesParams {
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Parameters for fetching funding history.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFundingHistoryParams {
    /// Filter by market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<i64>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}
