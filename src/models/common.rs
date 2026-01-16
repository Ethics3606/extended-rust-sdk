//! Common types used across the SDK.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

/// Helper to deserialize string numbers as Decimal.
fn decimal_from_string<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<Decimal>().map_err(serde::de::Error::custom)
}

/// Pagination parameters for cursor-based pagination.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PaginationParams {
    /// Cursor for pagination (ID to start from).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<i64>,
    /// Maximum number of items to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl PaginationParams {
    /// Create new pagination parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the cursor.
    pub fn with_cursor(mut self, cursor: i64) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Set the limit.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Pagination info returned in paginated responses.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationInfo {
    /// Next cursor for pagination.
    pub cursor: Option<i64>,
    /// Number of items returned.
    pub count: u32,
}

/// Wrapper for paginated API responses.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The data items.
    pub data: Vec<T>,
    /// Pagination information.
    pub pagination: PaginationInfo,
}

impl<T> PaginatedResponse<T> {
    /// Check if there are more pages available.
    pub fn has_more(&self) -> bool {
        self.pagination.cursor.is_some()
    }

    /// Get the next cursor if available.
    pub fn next_cursor(&self) -> Option<i64> {
        self.pagination.cursor
    }
}

/// Standard API response wrapper.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    /// Response status ("success" or "error").
    pub status: String,
    /// Response data (only present on success).
    pub data: Option<T>,
}

/// Price-quantity pair used in orderbooks.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriceQuantity {
    /// Price level.
    #[serde(deserialize_with = "decimal_from_string")]
    pub price: Decimal,
    /// Quantity at this price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub quantity: Decimal,
}

/// Time interval for candles and other time-series data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInterval {
    /// 1 minute interval
    #[serde(rename = "PT1M")]
    OneMinute,
    /// 5 minute interval
    #[serde(rename = "PT5M")]
    FiveMinutes,
    /// 15 minute interval
    #[serde(rename = "PT15M")]
    FifteenMinutes,
    /// 30 minute interval
    #[serde(rename = "PT30M")]
    ThirtyMinutes,
    /// 1 hour interval
    #[serde(rename = "PT1H")]
    OneHour,
    /// 4 hour interval
    #[serde(rename = "PT4H")]
    FourHours,
    /// 1 day interval
    #[serde(rename = "P1D")]
    OneDay,
    /// 1 week interval
    #[serde(rename = "P1W")]
    OneWeek,
}

impl TimeInterval {
    /// Get the string representation for API requests.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneMinute => "PT1M",
            Self::FiveMinutes => "PT5M",
            Self::FifteenMinutes => "PT15M",
            Self::ThirtyMinutes => "PT30M",
            Self::OneHour => "PT1H",
            Self::FourHours => "PT4H",
            Self::OneDay => "P1D",
            Self::OneWeek => "P1W",
        }
    }
}

/// Candle type for different price sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandleType {
    /// Trade prices
    Trades,
    /// Mark price
    Mark,
    /// Index price
    Index,
}

impl CandleType {
    /// Get the string representation for API requests.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trades => "trades",
            Self::Mark => "mark",
            Self::Index => "index",
        }
    }
}
