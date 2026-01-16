//! Public API endpoints (no authentication required).

use std::collections::HashMap;

use crate::client::HttpClient;
use crate::error::Result;
use crate::models::{
    Candle, CandleType, FundingRate, GetCandlesParams, GetPublicTradesParams,
    Market, MarketStats, OpenInterest, OrderBook, PublicTrade, TimeInterval,
};

/// Public API for Extended Exchange.
///
/// These endpoints do not require authentication and provide market data.
#[derive(Debug, Clone)]
pub struct PublicApi {
    client: HttpClient,
}

impl PublicApi {
    /// Create a new public API instance.
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    /// Get all available markets as a HashMap keyed by market name.
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> extended_rust_sdk::error::Result<()> {
    /// use extended_rust_sdk::{config::testnet_config, api::PublicApi, client::HttpClient};
    ///
    /// let client = HttpClient::new(testnet_config())?;
    /// let api = PublicApi::new(client);
    /// let markets = api.get_markets().await?;
    ///
    /// // Efficient lookup by name
    /// if let Some(btc) = markets.get("BTC-USD") {
    ///     println!("BTC tick size: {}", btc.config().tick_size());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_markets(&self) -> Result<HashMap<String, Market>> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<Market>,
        }
        let resp: Response = self.client.get("info/markets").await?;
        let map = resp.data.into_iter().map(|m| (m.name.clone(), m)).collect();
        Ok(map)
    }

    /// Get statistics for a specific market.
    ///
    /// # Arguments
    /// * `market` - Market name (e.g., "BTC-USD")
    pub async fn get_market_stats(&self, market: &str) -> Result<MarketStats> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: MarketStats,
        }
        let resp: Response = self.client.get(&format!("info/markets/{}/stats", market)).await?;
        Ok(resp.data)
    }

    /// Get order book for a market.
    ///
    /// # Arguments
    /// * `market` - Market name (e.g., "BTC-USD")
    /// * `depth` - Optional depth limit (default is full book)
    pub async fn get_orderbook(&self, market: &str, depth: Option<u32>) -> Result<OrderBook> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: OrderBook,
        }

        let path = if let Some(d) = depth {
            format!("info/markets/{}/orderbook?depth={}", market, d)
        } else {
            format!("info/markets/{}/orderbook", market)
        };

        let resp: Response = self.client.get(&path).await?;
        Ok(resp.data)
    }

    /// Get recent public trades for a market.
    ///
    /// # Arguments
    /// * `market` - Market name (e.g., "BTC-USD")
    /// * `params` - Optional parameters (limit)
    pub async fn get_trades(
        &self,
        market: &str,
        params: Option<GetPublicTradesParams>,
    ) -> Result<Vec<PublicTrade>> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<PublicTrade>,
        }

        let path = format!("info/markets/{}/trades", market);
        let resp: Response = if let Some(p) = params {
            self.client.get_with_query(&path, &p).await?
        } else {
            self.client.get(&path).await?
        };
        Ok(resp.data)
    }

    /// Get candlestick data for a market.
    ///
    /// # Arguments
    /// * `market` - Market name (e.g., "BTC-USD")
    /// * `candle_type` - Type of candle (trades, mark, or index)
    /// * `params` - Candle parameters (interval, time range, limit)
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> extended_rust_sdk::error::Result<()> {
    /// use extended_rust_sdk::{
    ///     config::testnet_config,
    ///     api::PublicApi,
    ///     client::HttpClient,
    ///     models::{CandleType, GetCandlesParams, TimeInterval},
    /// };
    ///
    /// let client = HttpClient::new(testnet_config())?;
    /// let api = PublicApi::new(client);
    /// let params = GetCandlesParams::new(TimeInterval::OneHour).with_limit(100);
    /// let candles = api.get_candles("BTC-USD", CandleType::Trades, params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_candles(
        &self,
        market: &str,
        candle_type: CandleType,
        params: GetCandlesParams,
    ) -> Result<Vec<Candle>> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<Candle>,
        }

        let path = format!(
            "info/candles/{}/{}/{}",
            market,
            candle_type.as_str(),
            params.interval.as_str()
        );

        let resp: Response = self.client.get_with_query(&path, &params).await?;
        Ok(resp.data)
    }

    /// Get funding rate history for a market.
    ///
    /// # Arguments
    /// * `market` - Market name (e.g., "BTC-USD")
    /// * `limit` - Optional limit on number of results
    pub async fn get_funding_rates(
        &self,
        market: &str,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<FundingRate>,
        }

        #[derive(serde::Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            limit: Option<u32>,
        }

        let path = format!("info/{}/funding", market);
        let resp: Response = self
            .client
            .get_with_query(&path, &Params { limit })
            .await?;
        Ok(resp.data)
    }

    /// Get open interest history for a market.
    ///
    /// # Arguments
    /// * `market` - Market name (e.g., "BTC-USD")
    /// * `interval` - Time interval for data points
    /// * `limit` - Optional limit on number of results
    pub async fn get_open_interest(
        &self,
        market: &str,
        interval: TimeInterval,
        limit: Option<u32>,
    ) -> Result<Vec<OpenInterest>> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<OpenInterest>,
        }

        #[derive(serde::Serialize)]
        struct Params {
            interval: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            limit: Option<u32>,
        }

        let path = format!("info/{}/open-interests", market);
        let resp: Response = self
            .client
            .get_with_query(
                &path,
                &Params {
                    interval: interval.as_str().to_string(),
                    limit,
                },
            )
            .await?;
        Ok(resp.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::testnet_config;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_markets() {
        let client = HttpClient::new(testnet_config()).unwrap();
        let api = PublicApi::new(client);
        let markets = api.get_markets().await.unwrap();
        assert!(!markets.is_empty());
    }
}
