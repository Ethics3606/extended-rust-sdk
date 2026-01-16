//! Order-related models.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

/// Default taker fee rate (0.05% = 5 basis points).
/// This is the standard fee tier. Use `get_fees()` to check your actual tier.
/// Value: 0.0005 = 5 × 10^-4
pub const DEFAULT_FEE_RATE: Decimal = Decimal::from_parts(5, 0, 0, false, 4);

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

/// Helper to deserialize id that can be either a string or an integer.
fn string_or_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i64),
        UInt(u64),
    }

    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::String(s) => Ok(s),
        StringOrInt::Int(i) => Ok(i.to_string()),
        StringOrInt::UInt(u) => Ok(u.to_string()),
    }
}

/// Order side (buy or sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    /// Buy/Long order.
    Buy,
    /// Sell/Short order.
    Sell,
}

impl OrderSide {
    /// Get the opposite side.
    pub fn opposite(&self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

/// Order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    /// Limit order.
    Limit,
    /// Market order.
    Market,
    /// Conditional order (stop order).
    Conditional,
    /// Take profit / stop loss order.
    Tpsl,
}

/// Time in force for orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    /// Good till time (default).
    #[serde(rename = "GTT")]
    GoodTillTime,
    /// Immediate or cancel.
    #[serde(rename = "IOC")]
    ImmediateOrCancel,
}

impl Default for TimeInForce {
    fn default() -> Self {
        Self::GoodTillTime
    }
}

/// Self-trade protection level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SelfTradeProtection {
    /// No self-trade protection.
    Disabled,
    /// Account-level protection (default).
    Account,
    /// Client-level protection.
    Client,
}

impl Default for SelfTradeProtection {
    fn default() -> Self {
        Self::Disabled
    }
}

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// Order is new (just created).
    New,
    /// Order is pending (not yet on book).
    Pending,
    /// Order is open on the book.
    Open,
    /// Order is partially filled.
    PartiallyFilled,
    /// Order is fully filled.
    Filled,
    /// Order was cancelled.
    Cancelled,
    /// Order was rejected.
    Rejected,
    /// Order expired.
    Expired,
}

impl OrderStatus {
    /// Check if the order is still active (can be filled or cancelled).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::New | Self::Pending | Self::Open | Self::PartiallyFilled)
    }

    /// Check if the order is terminal (no more changes possible).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Filled | Self::Cancelled | Self::Rejected | Self::Expired)
    }
}

/// Trigger type for conditional orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TriggerType {
    /// Trigger on mark price.
    Mark,
    /// Trigger on last trade price.
    Last,
    /// Trigger on index price.
    Index,
}

/// Order details.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Internal order ID (can be integer or string from API).
    #[serde(deserialize_with = "string_or_int")]
    pub id: String,
    /// Account ID.
    #[serde(default)]
    pub account_id: Option<i64>,
    /// External order ID (client-provided).
    #[serde(default)]
    pub external_id: Option<String>,
    /// Market name.
    pub market: String,
    /// Order side.
    pub side: OrderSide,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order status.
    pub status: OrderStatus,
    /// Order price.
    #[serde(deserialize_with = "decimal_from_string")]
    pub price: Decimal,
    /// Order quantity.
    #[serde(rename = "qty", deserialize_with = "decimal_from_string")]
    pub quantity: Decimal,
    /// Filled quantity.
    #[serde(default, rename = "filledQty", deserialize_with = "option_decimal_from_string")]
    pub filled_quantity: Option<Decimal>,
    /// Cancelled quantity.
    #[serde(default, rename = "cancelledQty", deserialize_with = "option_decimal_from_string")]
    pub cancelled_quantity: Option<Decimal>,
    /// Average fill price.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub average_price: Option<Decimal>,
    /// Time in force.
    #[serde(default)]
    pub time_in_force: Option<TimeInForce>,
    /// Whether this is a reduce-only order.
    #[serde(default)]
    pub reduce_only: Option<bool>,
    /// Whether this is a post-only order.
    #[serde(default)]
    pub post_only: Option<bool>,
    /// Trigger price for conditional orders.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub trigger_price: Option<Decimal>,
    /// Trigger type for conditional orders.
    #[serde(default)]
    pub trigger_type: Option<TriggerType>,
    /// Creation timestamp (Unix ms).
    #[serde(default, rename = "createdTime")]
    pub created_at: Option<i64>,
    /// Last update timestamp (Unix ms).
    #[serde(default, rename = "updatedTime")]
    pub updated_at: Option<i64>,
    /// Expiry timestamp (Unix ms).
    #[serde(default)]
    pub expire_time: Option<i64>,
    /// Fee paid.
    #[serde(default, rename = "payedFee", deserialize_with = "option_decimal_from_string")]
    pub paid_fee: Option<Decimal>,
}

impl Order {
    /// Get the filled quantity, defaulting to zero if not present.
    pub fn get_filled_quantity(&self) -> Decimal {
        self.filled_quantity.unwrap_or(Decimal::ZERO)
    }

    /// Get the unfilled quantity.
    pub fn unfilled_quantity(&self) -> Decimal {
        self.quantity - self.get_filled_quantity()
    }

    /// Check if the order is completely filled.
    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }
}

/// Request to create a new order.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    /// External order ID (derived from order hash or client-provided).
    pub id: String,
    /// Market name.
    pub market: String,
    /// Order side.
    pub side: OrderSide,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order price.
    pub price: Decimal,
    /// Order quantity (serialized as "qty" to match API).
    #[serde(rename = "qty")]
    pub quantity: Decimal,
    /// Whether this is a reduce-only order.
    #[serde(default)]
    pub reduce_only: bool,
    /// Whether this is a post-only order.
    #[serde(default)]
    pub post_only: bool,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Expiry time (Unix timestamp ms).
    pub expiry_epoch_millis: i64,
    /// Trading fee rate (taker fee as decimal, e.g., 0.0005 = 0.05% = 5 bps).
    /// Get your fee tier from `get_fees()`. Default taker rate is typically 0.0005.
    pub fee: Decimal,
    /// Nonce for Stark signature (as Decimal to match API format).
    pub nonce: Decimal,
    /// Self-trade protection level.
    pub self_trade_protection_level: SelfTradeProtection,
    /// Cancel ID for order replacement (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_id: Option<String>,
    /// Stark settlement data containing signature, public key, and vault ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement: Option<StarkSettlementModel>,
    /// Trigger configuration for conditional orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<ConditionalTrigger>,
    /// TPSL type (ORDER or POSITION).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tp_sl_type: Option<TpslType>,
    /// Take profit trigger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<TpslTrigger>,
    /// Stop loss trigger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<TpslTrigger>,
    /// Debugging amounts (optional, for troubleshooting).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debugging_amounts: Option<StarkDebuggingOrderAmounts>,
    /// Builder fee (optional, for builder integrations).
    #[serde(skip_serializing_if = "Option::is_none", rename = "builderFee")]
    pub builder_fee: Option<Decimal>,
    /// Builder ID (optional, for builder integrations).
    #[serde(skip_serializing_if = "Option::is_none", rename = "builderId")]
    pub builder_id: Option<i32>,
}

/// Conditional trigger configuration for stop/conditional orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalTrigger {
    /// Trigger price.
    pub trigger_price: Decimal,
    /// Trigger price type (mark, last, index).
    pub trigger_price_type: TriggerType,
    /// Trigger direction (up or down).
    pub direction: TriggerDirection,
    /// Execution price type.
    pub execution_price_type: OrderPriceType,
}

/// Trigger direction for conditional orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TriggerDirection {
    /// Trigger when price goes up.
    Up,
    /// Trigger when price goes down.
    Down,
}

/// Order price type for execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderPriceType {
    /// Market price.
    Market,
    /// Limit price.
    Limit,
}

/// TPSL type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TpslType {
    /// Order-based TPSL.
    Order,
    /// Position-based TPSL.
    Position,
}

/// Take profit or stop loss trigger configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TpslTrigger {
    /// Trigger price.
    pub trigger_price: Decimal,
    /// Trigger price type.
    pub trigger_price_type: TriggerType,
    /// Execution price.
    pub price: Decimal,
    /// Price type.
    pub price_type: OrderPriceType,
    /// Settlement data for this trigger.
    pub settlement: StarkSettlementModel,
    /// Debugging amounts (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debugging_amounts: Option<StarkDebuggingOrderAmounts>,
}

/// Stark signature for orders (r and s components as hex strings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementSignature {
    /// Signature r component (hex string).
    pub r: String,
    /// Signature s component (hex string).
    pub s: String,
}

/// Alias for backwards compatibility.
pub type OrderSignature = SettlementSignature;

/// Stark settlement model containing signature and account info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarkSettlementModel {
    /// Stark signature (r, s components).
    pub signature: SettlementSignature,
    /// Stark public key (hex string).
    pub stark_key: String,
    /// Collateral position ID (vault ID as Decimal).
    pub collateral_position: Decimal,
}

/// Debugging amounts for order (optional, for troubleshooting).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarkDebuggingOrderAmounts {
    /// Collateral amount in stark units.
    pub collateral_amount: Decimal,
    /// Fee amount in stark units.
    pub fee_amount: Decimal,
    /// Synthetic amount in stark units.
    pub synthetic_amount: Decimal,
}

/// Builder for creating order requests.
#[derive(Debug, Clone)]
pub struct OrderBuilder {
    market: String,
    side: OrderSide,
    order_type: OrderType,
    price: Decimal,
    quantity: Decimal,
    fee: Decimal,
    nonce: Option<u64>,
    time_in_force: TimeInForce,
    reduce_only: bool,
    post_only: bool,
    external_id: Option<String>,
    trigger_price: Option<Decimal>,
    trigger_type: Option<TriggerType>,
    expiry_epoch_millis: Option<i64>,
    self_trade_protection: SelfTradeProtection,
}

impl OrderBuilder {
    /// Create a new limit order builder.
    ///
    /// # Arguments
    /// * `market` - Market name (e.g., "BTC-USD")
    /// * `side` - Buy or Sell
    /// * `price` - Limit price
    /// * `quantity` - Order quantity
    /// * `post_only` - If true, order will only add liquidity (maker only)
    /// * `reduce_only` - If true, order can only reduce an existing position
    pub fn limit(
        market: impl Into<String>,
        side: OrderSide,
        price: Decimal,
        quantity: Decimal,
        post_only: bool,
        reduce_only: bool,
    ) -> Self {
        Self {
            market: market.into(),
            side,
            order_type: OrderType::Limit,
            price,
            quantity,
            fee: DEFAULT_FEE_RATE,
            nonce: None,
            time_in_force: TimeInForce::GoodTillTime,
            reduce_only,
            post_only,
            external_id: None,
            trigger_price: None,
            trigger_type: None,
            expiry_epoch_millis: None,
            self_trade_protection: SelfTradeProtection::Disabled,
        }
    }

    /// Set time in force.
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = tif;
        self
    }

    /// Set reduce only flag.
    pub fn reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = reduce_only;
        self
    }

    /// Set post only flag.
    pub fn post_only(mut self, post_only: bool) -> Self {
        self.post_only = post_only;
        self
    }

    /// Set external ID.
    pub fn external_id(mut self, id: impl Into<String>) -> Self {
        self.external_id = Some(id.into());
        self
    }

    /// Set trigger price for conditional orders.
    pub fn trigger(mut self, price: Decimal, trigger_type: TriggerType) -> Self {
        self.trigger_price = Some(price);
        self.trigger_type = Some(trigger_type);
        self.order_type = OrderType::Conditional;
        self
    }

    /// Set expiry time.
    pub fn expiry(mut self, expiry_millis: i64) -> Self {
        self.expiry_epoch_millis = Some(expiry_millis);
        self
    }

    /// Set self-trade protection level.
    pub fn self_trade_protection(mut self, level: SelfTradeProtection) -> Self {
        self.self_trade_protection = level;
        self
    }

    /// Override the fee rate (default is DEFAULT_FEE_RATE = 0.0005).
    /// Use your tier's taker rate from `get_fees()` if different.
    pub fn fee(mut self, fee: Decimal) -> Self {
        self.fee = fee;
        self
    }

    /// Override the nonce (default is auto-generated from current timestamp).
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Build the order request (without settlement - must be signed separately).
    ///
    /// Nonce is auto-generated from current timestamp if not set via `.nonce()`.
    /// Fee defaults to DEFAULT_FEE_RATE (0.0005) if not set via `.fee()`.
    /// Expiry defaults to 1 hour from now if not set via `.expiry()`.
    /// The `id` field is set to the nonce as string (will be replaced with order hash after signing).
    pub fn build(self) -> CreateOrderRequest {
        let nonce = self.nonce.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time before UNIX epoch")
                .as_millis() as u64
        });

        // Default expiry is 1 hour from now
        let expiry = self.expiry_epoch_millis.unwrap_or_else(|| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time before UNIX epoch")
                .as_millis() as i64;
            now + 3600 * 1000 // 1 hour in milliseconds
        });

        // Use external_id if provided, otherwise use nonce as temporary ID
        // (will be replaced with order hash after signing)
        let id = self.external_id.clone().unwrap_or_else(|| nonce.to_string());

        CreateOrderRequest {
            id,
            market: self.market,
            side: self.side,
            order_type: self.order_type,
            price: self.price,
            quantity: self.quantity,
            reduce_only: self.reduce_only,
            post_only: self.post_only,
            time_in_force: self.time_in_force,
            expiry_epoch_millis: expiry,
            fee: self.fee,
            nonce: Decimal::from(nonce),
            self_trade_protection_level: self.self_trade_protection,
            cancel_id: None,
            settlement: None,
            trigger: None,
            tp_sl_type: None,
            take_profit: None,
            stop_loss: None,
            debugging_amounts: None,
            builder_fee: None,
            builder_id: None,
        }
    }
}

/// Parameters for cancelling orders.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MassCancelParams {
    /// Market to cancel orders for (optional, all markets if not specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    /// Side to cancel (optional, both sides if not specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<OrderSide>,
}

/// Response from mass cancel operation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MassCancelResponse {
    /// Number of orders cancelled.
    pub cancelled_count: u32,
}

/// Response from placing an order.
///
/// The API only returns the order IDs on creation. Use `get_order()` to fetch full details.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedOrderResponse {
    /// Internal order ID (can be integer or string from API).
    #[serde(deserialize_with = "string_or_int")]
    pub id: String,
    /// External order ID (client-provided, derived from order hash).
    pub external_id: String,
}

/// Parameters for fetching orders.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrdersParams {
    /// Filter by market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    /// Filter by side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<OrderSide>,
    /// Filter by status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<i64>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}
