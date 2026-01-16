//! Candlestick (OHLCV) models.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use super::TimeInterval;

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

/// OHLCV candlestick data.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candle {
    /// Candle open time (Unix ms).
    pub timestamp: i64,
    /// Open price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub open: Decimal,
    /// High price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub high: Decimal,
    /// Low price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub low: Decimal,
    /// Close price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub close: Decimal,
    /// Trading volume in base asset.
    #[serde(deserialize_with = "decimal_from_string")]
    pub volume: Decimal,
    /// Trading volume in quote asset.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub quote_volume: Option<Decimal>,
    /// Number of trades.
    #[serde(default)]
    pub trades: Option<u64>,
}

impl Candle {
    /// Check if the candle is bullish (close > open).
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Check if the candle is bearish (close < open).
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Get the candle body size.
    pub fn body(&self) -> Decimal {
        (self.close - self.open).abs()
    }

    /// Get the upper wick size.
    pub fn upper_wick(&self) -> Decimal {
        if self.is_bullish() {
            self.high - self.close
        } else {
            self.high - self.open
        }
    }

    /// Get the lower wick size.
    pub fn lower_wick(&self) -> Decimal {
        if self.is_bullish() {
            self.open - self.low
        } else {
            self.close - self.low
        }
    }

    /// Get the full range (high - low).
    pub fn range(&self) -> Decimal {
        self.high - self.low
    }

    /// Get the typical price ((high + low + close) / 3).
    pub fn typical_price(&self) -> Decimal {
        (self.high + self.low + self.close) / Decimal::from(3)
    }

    /// Get the VWAP if quote volume is available.
    pub fn vwap(&self) -> Option<Decimal> {
        self.quote_volume.map(|qv| {
            if self.volume.is_zero() {
                self.close
            } else {
                qv / self.volume
            }
        })
    }
}

/// Parameters for fetching candles.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCandlesParams {
    /// Time interval for candles.
    #[serde(skip)]
    pub interval: TimeInterval,
    /// Start timestamp (Unix ms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// End timestamp (Unix ms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// Maximum number of candles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl Default for GetCandlesParams {
    fn default() -> Self {
        Self {
            interval: TimeInterval::OneHour,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }
}

impl GetCandlesParams {
    /// Create parameters for a specific interval.
    pub fn new(interval: TimeInterval) -> Self {
        Self {
            interval,
            ..Default::default()
        }
    }

    /// Set the time range.
    pub fn with_range(mut self, start: i64, end: i64) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Set the limit.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}
