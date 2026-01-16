//! Private API endpoints (authentication required).

use crate::client::HttpClient;
use crate::error::Result;
use crate::models::{
    AccountInfo, AssetOperation, Balance, CreateOrderRequest, MarketFee,
    FundingPayment, GetFundingHistoryParams, GetOrdersParams,
    GetPositionHistoryParams, GetPositionsParams, GetTradesParams, Leverage,
    MassCancelParams, MassCancelResponse, Order, PaginatedResponse, PlacedOrderResponse,
    Position, PositionHistory, SpotBalance, SpotBalances, Trade, Transfer, TransferRequest,
    UpdateLeverageRequest, Withdrawal, WithdrawalRequest,
};

/// Private API for Extended Exchange.
///
/// These endpoints require authentication via API key.
#[derive(Debug, Clone)]
pub struct PrivateApi {
    client: HttpClient,
}

impl PrivateApi {
    /// Create a new private API instance.
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    // ========== Account Endpoints ==========

    /// Get account information.
    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: AccountInfo,
        }
        let resp: Response = self.client.get("user/account/info").await?;
        Ok(resp.data)
    }

    /// Get account balance.
    pub async fn get_balance(&self) -> Result<Balance> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Balance,
        }
        let resp: Response = self.client.get("user/balance").await?;
        Ok(resp.data)
    }

    /// Get spot/collateral balances with full breakdown.
    ///
    /// Returns individual asset balances including:
    /// - Raw balance amount
    /// - Index price
    /// - Notional value (balance * index_price)
    /// - Contribution factor (e.g., 0.9 for XVS vault shares)
    /// - Equity contribution (notional * contribution_factor)
    ///
    /// Use `SpotBalances::total_notional_value()` to get the true USD value
    /// before contribution factors are applied.
    pub async fn get_spot_balances(&self) -> Result<SpotBalances> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<SpotBalance>,
        }
        let resp: Response = self.client.get("user/spot/balances").await?;
        Ok(SpotBalances::from(resp.data))
    }

    /// Get asset operations history (deposits, withdrawals, transfers).
    ///
    /// # Arguments
    /// * `cursor` - Optional pagination cursor
    /// * `limit` - Optional limit on results
    pub async fn get_asset_operations(
        &self,
        cursor: Option<i64>,
        limit: Option<u32>,
    ) -> Result<PaginatedResponse<AssetOperation>> {
        #[derive(serde::Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            cursor: Option<i64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            limit: Option<u32>,
        }

        self.client
            .get_with_query("user/assetOperations", &Params { cursor, limit })
            .await
    }

    /// Get fee structure for all markets.
    pub async fn get_fees(&self) -> Result<Vec<MarketFee>> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<MarketFee>,
        }
        let resp: Response = self.client.get("user/fees").await?;
        Ok(resp.data)
    }

    // ========== Position Endpoints ==========

    /// Get open positions.
    ///
    /// # Arguments
    /// * `params` - Optional filter parameters
    pub async fn get_positions(&self, params: Option<GetPositionsParams>) -> Result<Vec<Position>> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<Position>,
        }

        let resp: Response = if let Some(p) = params {
            self.client.get_with_query("user/positions", &p).await?
        } else {
            self.client.get("user/positions").await?
        };
        Ok(resp.data)
    }

    /// Get position history.
    ///
    /// # Arguments
    /// * `params` - Optional filter and pagination parameters
    pub async fn get_position_history(
        &self,
        params: Option<GetPositionHistoryParams>,
    ) -> Result<PaginatedResponse<PositionHistory>> {
        let params = params.unwrap_or_default();
        self.client
            .get_with_query("user/positions/history", &params)
            .await
    }

    // ========== Leverage Endpoints ==========

    /// Get current leverage settings.
    ///
    /// # Arguments
    /// * `market` - Optional market filter
    pub async fn get_leverage(&self, market: Option<&str>) -> Result<Vec<Leverage>> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<Leverage>,
        }

        #[derive(serde::Serialize)]
        struct Params<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            market: Option<&'a str>,
        }

        let resp: Response = self
            .client
            .get_with_query("user/leverage", &Params { market })
            .await?;
        Ok(resp.data)
    }

    /// Update leverage for a market.
    ///
    /// # Arguments
    /// * `market` - Market name
    /// * `leverage` - New leverage value
    pub async fn update_leverage(&self, market: &str, leverage: u32) -> Result<Leverage> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Leverage,
        }

        let req = UpdateLeverageRequest {
            market: market.to_string(),
            leverage,
        };

        let resp: Response = self.client.patch("user/leverage", &req).await?;
        Ok(resp.data)
    }

    // ========== Order Endpoints ==========

    /// Create a new order.
    ///
    /// # Arguments
    /// * `request` - Order creation request (must be signed)
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> extended_rust_sdk::error::Result<()> {
    /// use rust_decimal_macros::dec;
    /// use extended_rust_sdk::{
    ///     config::testnet_config,
    ///     api::PrivateApi,
    ///     client::HttpClient,
    ///     models::{OrderBuilder, OrderSide},
    /// };
    ///
    /// let client = HttpClient::with_api_key(testnet_config(), "your-api-key")?;
    /// let api = PrivateApi::new(client);
    ///
    /// // Build order (must sign before submitting)
    /// let order = OrderBuilder::limit("BTC-USD", OrderSide::Buy, dec!(50000), dec!(0.01))
    ///     .post_only(true)
    ///     .build(dec!(0.0001), 1);
    ///
    /// // Note: Order needs to be signed before submission
    /// // let signed_order = sign_order(order, &stark_account);
    /// // let result = api.create_order(signed_order).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_order(&self, request: CreateOrderRequest) -> Result<PlacedOrderResponse> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: PlacedOrderResponse,
        }

        let resp: Response = self.client.post("user/order", &request).await?;
        Ok(resp.data)
    }

    /// Cancel an order by internal ID.
    ///
    /// # Arguments
    /// * `order_id` - Internal order ID
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        #[derive(serde::Deserialize)]
        struct Response {
            #[allow(dead_code)]
            status: String,
        }

        let _: Response = self
            .client
            .delete(&format!("user/order/{}", order_id))
            .await?;
        Ok(())
    }

    /// Cancel an order by external ID.
    ///
    /// # Arguments
    /// * `external_id` - External order ID (client-provided)
    pub async fn cancel_order_by_external_id(&self, external_id: &str) -> Result<()> {
        #[derive(serde::Deserialize)]
        struct Response {
            #[allow(dead_code)]
            status: String,
        }

        #[derive(serde::Serialize)]
        struct Params<'a> {
            #[serde(rename = "externalId")]
            external_id: &'a str,
        }

        let _: Response = self
            .client
            .delete_with_query("user/order", &Params { external_id })
            .await?;
        Ok(())
    }

    /// Mass cancel orders.
    ///
    /// # Arguments
    /// * `params` - Optional filter parameters (market, side)
    pub async fn mass_cancel(&self, params: Option<MassCancelParams>) -> Result<MassCancelResponse> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: MassCancelResponse,
        }

        let resp: Response = if let Some(p) = params {
            self.client.post("user/order/massCancel", &p).await?
        } else {
            self.client.post_empty("user/order/massCancel").await?
        };
        Ok(resp.data)
    }

    /// Get open orders.
    ///
    /// # Arguments
    /// * `params` - Optional filter parameters
    pub async fn get_open_orders(&self, params: Option<GetOrdersParams>) -> Result<Vec<Order>> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<Order>,
        }

        let resp: Response = if let Some(p) = params {
            self.client.get_with_query("user/orders", &p).await?
        } else {
            self.client.get("user/orders").await?
        };
        Ok(resp.data)
    }

    /// Get order history.
    ///
    /// # Arguments
    /// * `params` - Optional filter and pagination parameters
    pub async fn get_orders_history(
        &self,
        params: Option<GetOrdersParams>,
    ) -> Result<PaginatedResponse<Order>> {
        let params = params.unwrap_or_default();
        self.client
            .get_with_query("user/orders/history", &params)
            .await
    }

    /// Get order by internal ID.
    ///
    /// # Arguments
    /// * `order_id` - Internal order ID
    pub async fn get_order(&self, order_id: &str) -> Result<Order> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Order,
        }

        let resp: Response = self
            .client
            .get(&format!("user/orders/{}", order_id))
            .await?;
        Ok(resp.data)
    }

    /// Get order by external ID.
    ///
    /// # Arguments
    /// * `external_id` - External order ID (client-provided)
    pub async fn get_order_by_external_id(&self, external_id: &str) -> Result<Order> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Order,
        }

        let resp: Response = self
            .client
            .get(&format!("user/orders/external/{}", external_id))
            .await?;
        Ok(resp.data)
    }

    // ========== Trade Endpoints ==========

    /// Get trade history (fills).
    ///
    /// # Arguments
    /// * `params` - Optional filter and pagination parameters
    pub async fn get_trades(
        &self,
        params: Option<GetTradesParams>,
    ) -> Result<PaginatedResponse<Trade>> {
        let params = params.unwrap_or_default();
        self.client.get_with_query("user/trades", &params).await
    }

    /// Get funding payment history.
    ///
    /// # Arguments
    /// * `params` - Optional filter and pagination parameters
    pub async fn get_funding_history(
        &self,
        params: Option<GetFundingHistoryParams>,
    ) -> Result<PaginatedResponse<FundingPayment>> {
        let params = params.unwrap_or_default();
        self.client
            .get_with_query("user/funding/history", &params)
            .await
    }

    // ========== Dead Man's Switch ==========

    /// Set dead man's switch countdown.
    ///
    /// When set, all orders will be automatically cancelled if the countdown
    /// expires without being refreshed.
    ///
    /// # Arguments
    /// * `countdown_seconds` - Countdown time in seconds (0 to disable)
    pub async fn set_dead_man_switch(&self, countdown_seconds: u32) -> Result<()> {
        #[derive(serde::Serialize)]
        struct Params {
            #[serde(rename = "countdownTime")]
            countdown_time: u32,
        }

        let _: serde_json::Value = self
            .client
            .post(
                &format!("user/deadmanswitch?countdownTime={}", countdown_seconds),
                &Params {
                    countdown_time: countdown_seconds,
                },
            )
            .await?;
        Ok(())
    }

    // ========== Withdrawal & Transfer Endpoints ==========

    /// Request a withdrawal.
    ///
    /// # Arguments
    /// * `request` - Withdrawal request (must be signed)
    pub async fn withdraw(&self, request: WithdrawalRequest) -> Result<Withdrawal> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Withdrawal,
        }

        let resp: Response = self.client.post("user/withdrawal", &request).await?;
        Ok(resp.data)
    }

    /// Transfer funds between sub-accounts.
    ///
    /// # Arguments
    /// * `request` - Transfer request (must be signed)
    pub async fn transfer(&self, request: TransferRequest) -> Result<Transfer> {
        #[derive(serde::Deserialize)]
        struct Response {
            data: Transfer,
        }

        let resp: Response = self.client.post("user/transfer", &request).await?;
        Ok(resp.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::testnet_config;

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_get_balance() {
        let client = HttpClient::with_api_key(testnet_config(), "test-api-key").unwrap();
        let api = PrivateApi::new(client);
        let balance = api.get_balance().await.unwrap();
        println!("Balance: {:?}", balance);
    }
}
