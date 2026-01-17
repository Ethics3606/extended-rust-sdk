//! Position-related models.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

/// Position side (Long or Short).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionSide {
    /// Long position.
    Long,
    /// Short position.
    Short,
}

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

/// Open position.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Position ID.
    #[serde(default)]
    pub id: Option<i64>,
    /// Market name.
    pub market: String,
    /// Position side (Long or Short).
    pub side: PositionSide,
    /// Position size.
    #[serde(deserialize_with = "decimal_from_string")]
    pub size: Decimal,
    /// Average entry price (API field: openPrice).
    #[serde(rename = "openPrice", deserialize_with = "decimal_from_string")]
    pub entry_price: Decimal,
    /// Current mark price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub mark_price: Decimal,
    /// Liquidation price.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub liquidation_price: Option<Decimal>,
    /// Unrealized PnL (API uses British spelling: unrealisedPnl).
    #[serde(rename = "unrealisedPnl", deserialize_with = "decimal_from_string")]
    pub unrealized_pnl: Decimal,
    /// Realized PnL (API uses British spelling: realisedPnl).
    #[serde(default, rename = "realisedPnl", deserialize_with = "option_decimal_from_string")]
    pub realized_pnl: Option<Decimal>,
    /// Position margin.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub margin: Option<Decimal>,
    /// Position notional value.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub value: Option<Decimal>,
    /// Position leverage.
    #[serde(deserialize_with = "decimal_from_string")]
    pub leverage: Decimal,
    /// Auto-deleveraging rank.
    #[serde(default)]
    pub adl: Option<i32>,
    /// Position created timestamp.
    #[serde(default)]
    pub created_at: Option<i64>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at: Option<i64>,
}

impl Position {
    /// Check if this is a long position.
    pub fn is_long(&self) -> bool {
        self.side == PositionSide::Long
    }

    /// Check if this is a short position.
    pub fn is_short(&self) -> bool {
        self.side == PositionSide::Short
    }

    /// Get margin, defaulting to zero if not present.
    pub fn get_margin(&self) -> Decimal {
        self.margin.unwrap_or(Decimal::ZERO)
    }

    /// Calculate ROE (Return on Equity).
    pub fn roe(&self) -> Decimal {
        let margin = self.get_margin();
        if margin.is_zero() {
            Decimal::ZERO
        } else {
            self.unrealized_pnl / margin * Decimal::from(100)
        }
    }

    /// Calculate PnL percentage.
    pub fn pnl_percentage(&self) -> Decimal {
        if self.entry_price.is_zero() || self.size.is_zero() {
            return Decimal::ZERO;
        }

        let entry_notional = self.size * self.entry_price;
        self.unrealized_pnl / entry_notional.abs() * Decimal::from(100)
    }
}

/// Historical position (closed).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionHistory {
    /// Position ID.
    pub id: String,
    /// Market name.
    pub market: String,
    /// Position side.
    pub side: PositionSide,
    /// Maximum position size.
    pub max_size: Decimal,
    /// Average entry price.
    pub entry_price: Decimal,
    /// Average exit price.
    pub exit_price: Decimal,
    /// Realized PnL.
    pub realized_pnl: Decimal,
    /// Accumulated funding payments.
    pub accumulated_funding: Decimal,
    /// Total fees paid.
    pub fees: Decimal,
    /// Position open timestamp.
    pub opened_at: i64,
    /// Position close timestamp.
    pub closed_at: i64,
    /// Close reason.
    pub close_reason: PositionCloseReason,
}

/// Reason for position closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionCloseReason {
    /// Closed by user (reduce order).
    User,
    /// Closed by liquidation.
    Liquidation,
    /// Closed by ADL (auto-deleveraging).
    Adl,
    /// Closed by settlement.
    Settlement,
}

impl PositionHistory {
    /// Calculate net PnL (realized - fees + funding).
    pub fn net_pnl(&self) -> Decimal {
        self.realized_pnl - self.fees + self.accumulated_funding
    }
}

/// Parameters for fetching positions.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPositionsParams {
    /// Filter by market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
}

/// Parameters for fetching position history.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPositionHistoryParams {
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
