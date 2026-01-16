//! Account and balance models.

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

/// Helper to deserialize optional string numbers as Option<Decimal>.
fn option_decimal_from_string<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => s.parse::<Decimal>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}


/// API key information (when returned as full object).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyInfo {
    /// API key value.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Key description.
    #[serde(default)]
    pub description: Option<String>,
    /// Permissions enabled for this key.
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
    /// Created timestamp.
    #[serde(default)]
    pub created_at: Option<i64>,
    /// Allow any other fields we don't know about.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Helper to deserialize optional api_keys array.
fn deserialize_api_keys<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // The API returns api_keys as an array of strings (the key values)
    // Handle both missing and present cases
    let opt: Option<Vec<String>> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Account information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// Account ID (numeric).
    #[serde(default)]
    pub account_id: Option<i64>,
    /// Account description.
    #[serde(default)]
    pub description: Option<String>,
    /// Account index (0 for main account).
    #[serde(default)]
    pub account_index: Option<i64>,
    /// Account status.
    #[serde(default)]
    pub status: Option<AccountStatus>,
    /// L2 (Stark) public key.
    #[serde(default)]
    pub l2_key: Option<String>,
    /// L2 vault ID.
    #[serde(default)]
    pub l2_vault: Option<String>,
    /// Bridge Starknet address.
    #[serde(default)]
    pub bridge_starknet_address: Option<String>,
    /// API keys associated with this account (as key strings).
    #[serde(default, deserialize_with = "deserialize_api_keys")]
    pub api_keys: Vec<String>,
    /// Account index used for key generation.
    #[serde(default)]
    pub account_index_for_key_generation: Option<i64>,
    /// Allow any other fields we don't know about.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl AccountInfo {
    /// Get account ID, defaulting to 0 if not present.
    pub fn get_account_id(&self) -> i64 {
        self.account_id.unwrap_or(0)
    }

    /// Get description, defaulting to empty string if not present.
    pub fn get_description(&self) -> String {
        self.description.clone().unwrap_or_default()
    }

    /// Get account index, defaulting to 0 if not present.
    pub fn get_account_index(&self) -> i64 {
        self.account_index.unwrap_or(0)
    }

    /// Get L2 key, defaulting to empty string if not present.
    pub fn get_l2_key(&self) -> String {
        self.l2_key.clone().unwrap_or_default()
    }

    /// Get L2 vault, defaulting to empty string if not present.
    pub fn get_l2_vault(&self) -> String {
        self.l2_vault.clone().unwrap_or_default()
    }
}

/// Account status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
    /// Account is active.
    Active,
    /// Account is suspended.
    Suspended,
    /// Account is being liquidated.
    Liquidating,
}

/// Account balance information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    /// Collateral name (e.g., "USD").
    #[serde(default)]
    pub collateral_name: Option<String>,
    /// Account balance (deposits - withdrawals + realized PnL).
    #[serde(deserialize_with = "decimal_from_string")]
    pub balance: Decimal,
    /// Account status.
    #[serde(default)]
    pub status: Option<String>,
    /// Total equity (balance + unrealized PnL).
    #[serde(deserialize_with = "decimal_from_string")]
    pub equity: Decimal,
    /// Spot equity.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub spot_equity: Option<Decimal>,
    /// Unrealized profit/loss.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub unrealized_pnl: Option<Decimal>,
    /// Total initial margin requirement.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub initial_margin: Option<Decimal>,
    /// Total maintenance margin requirement.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub maintenance_margin: Option<Decimal>,
    /// Available for trading (equity - initial margin).
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub available_for_trade: Option<Decimal>,
    /// Available for withdrawal.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub available_for_withdrawal: Option<Decimal>,
    /// Account margin ratio (maintenance margin / equity).
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub margin_ratio: Option<Decimal>,
    /// Account leverage (total exposure / equity).
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub account_leverage: Option<Decimal>,
    /// Total exposure (sum of position notional values).
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub total_exposure: Option<Decimal>,
}

impl Balance {
    /// Get account balance (alias for balance field).
    pub fn account_balance(&self) -> Decimal {
        self.balance
    }

    /// Get unrealized PnL, defaulting to zero if not present.
    pub fn get_unrealized_pnl(&self) -> Decimal {
        self.unrealized_pnl.unwrap_or(Decimal::ZERO)
    }

    /// Get initial margin, defaulting to zero if not present.
    pub fn get_initial_margin(&self) -> Decimal {
        self.initial_margin.unwrap_or(Decimal::ZERO)
    }

    /// Get maintenance margin, defaulting to zero if not present.
    pub fn get_maintenance_margin(&self) -> Decimal {
        self.maintenance_margin.unwrap_or(Decimal::ZERO)
    }

    /// Get available for trade, defaulting to zero if not present.
    pub fn get_available_for_trade(&self) -> Decimal {
        self.available_for_trade.unwrap_or(Decimal::ZERO)
    }

    /// Get available for withdrawal, defaulting to zero if not present.
    pub fn get_available_for_withdrawal(&self) -> Decimal {
        self.available_for_withdrawal.unwrap_or(Decimal::ZERO)
    }

    /// Get margin ratio, defaulting to zero if not present.
    pub fn get_margin_ratio(&self) -> Decimal {
        self.margin_ratio.unwrap_or(Decimal::ZERO)
    }

    /// Get account leverage, defaulting to zero if not present.
    pub fn get_account_leverage(&self) -> Decimal {
        self.account_leverage.unwrap_or(Decimal::ZERO)
    }

    /// Check if the account is at risk of liquidation.
    pub fn is_at_risk(&self) -> bool {
        self.get_margin_ratio() >= Decimal::from(80) / Decimal::from(100)
    }

    /// Check if the account is being liquidated.
    pub fn is_liquidating(&self) -> bool {
        self.get_margin_ratio() >= Decimal::ONE
    }
}

/// Leverage configuration per market.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Leverage {
    /// Market name.
    pub market: String,
    /// Current leverage multiplier (can be decimal like "5.00").
    #[serde(deserialize_with = "decimal_from_string")]
    pub leverage: Decimal,
    /// Maximum allowed leverage for this market.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub max_leverage: Option<Decimal>,
}

impl Leverage {
    /// Get leverage as integer (truncated).
    pub fn leverage_int(&self) -> u32 {
        use rust_decimal::prelude::ToPrimitive;
        self.leverage.to_u32().unwrap_or(1)
    }

    /// Get max leverage as integer (truncated).
    pub fn max_leverage_int(&self) -> Option<u32> {
        use rust_decimal::prelude::ToPrimitive;
        self.max_leverage.and_then(|d| d.to_u32())
    }
}


/// Request to update leverage.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLeverageRequest {
    /// Market name.
    pub market: String,
    /// New leverage value.
    pub leverage: u32,
}

/// Per-market fee structure (API returns array of these).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketFee {
    /// Market name.
    #[serde(default)]
    pub market: Option<String>,
    /// Maker fee rate.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub maker_fee_rate: Option<Decimal>,
    /// Taker fee rate.
    #[serde(default, deserialize_with = "option_decimal_from_string")]
    pub taker_fee_rate: Option<Decimal>,
    /// Allow any other fields we don't know about.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl MarketFee {
    /// Get market name, defaulting to empty string.
    pub fn get_market(&self) -> String {
        self.market.clone().unwrap_or_default()
    }

    /// Get maker fee rate, defaulting to zero.
    pub fn get_maker_fee_rate(&self) -> Decimal {
        self.maker_fee_rate.unwrap_or(Decimal::ZERO)
    }

    /// Get taker fee rate, defaulting to zero.
    pub fn get_taker_fee_rate(&self) -> Decimal {
        self.taker_fee_rate.unwrap_or(Decimal::ZERO)
    }
}

/// Legacy Fees type alias for backwards compatibility.
pub type Fees = MarketFee;

/// Individual spot/collateral balance for an asset.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotBalance {
    /// Account ID.
    #[serde(default)]
    pub account_id: Option<i64>,
    /// Asset name (e.g., "USD", "XVS").
    pub asset: String,
    /// Raw balance amount.
    #[serde(deserialize_with = "decimal_from_string")]
    pub balance: Decimal,
    /// Index price of the asset.
    #[serde(deserialize_with = "decimal_from_string")]
    pub index_price: Decimal,
    /// Notional value in USD (balance * index_price).
    #[serde(deserialize_with = "decimal_from_string")]
    pub notional_value: Decimal,
    /// Contribution factor (e.g., 1.0 for USD, 0.9 for XVS).
    #[serde(deserialize_with = "decimal_from_string")]
    pub contribution_factor: Decimal,
    /// Equity contribution (notional_value * contribution_factor).
    #[serde(deserialize_with = "decimal_from_string")]
    pub equity_contribution: Decimal,
    /// Amount available to withdraw.
    #[serde(deserialize_with = "decimal_from_string")]
    pub available_to_withdraw: Decimal,
    /// Last update timestamp (Unix ms).
    #[serde(default)]
    pub updated_at: Option<i64>,
}

impl SpotBalance {
    /// Check if this is a stablecoin (USD, USDC, etc.) with 100% contribution.
    pub fn is_stablecoin(&self) -> bool {
        self.contribution_factor == Decimal::ONE
    }

    /// Get the "haircut" amount (notional - equity contribution).
    pub fn haircut(&self) -> Decimal {
        self.notional_value - self.equity_contribution
    }
}

/// Collection of spot balances with helper methods.
#[derive(Debug, Clone)]
pub struct SpotBalances(pub Vec<SpotBalance>);

impl SpotBalances {
    /// Get total notional value (true USD value before contribution factors).
    pub fn total_notional_value(&self) -> Decimal {
        self.0.iter().map(|b| b.notional_value).sum()
    }

    /// Get total equity contribution (after contribution factors applied).
    pub fn total_equity_contribution(&self) -> Decimal {
        self.0.iter().map(|b| b.equity_contribution).sum()
    }

    /// Get total haircut amount.
    pub fn total_haircut(&self) -> Decimal {
        self.total_notional_value() - self.total_equity_contribution()
    }

    /// Find balance for a specific asset.
    pub fn get(&self, asset: &str) -> Option<&SpotBalance> {
        self.0.iter().find(|b| b.asset == asset)
    }

    /// Iterate over all balances.
    pub fn iter(&self) -> impl Iterator<Item = &SpotBalance> {
        self.0.iter()
    }

    /// Number of assets.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<SpotBalance>> for SpotBalances {
    fn from(v: Vec<SpotBalance>) -> Self {
        Self(v)
    }
}

/// Asset operation (deposit, withdrawal, transfer).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetOperation {
    /// Operation ID.
    pub id: String,
    /// Operation type.
    #[serde(default)]
    pub operation_type: Option<AssetOperationType>,
    /// Asset (usually "USDC").
    #[serde(default)]
    pub asset: Option<String>,
    /// Amount.
    #[serde(deserialize_with = "decimal_from_string")]
    pub amount: Decimal,
    /// Operation status.
    #[serde(default)]
    pub status: Option<AssetOperationStatus>,
    /// Transaction hash (if applicable).
    #[serde(default)]
    pub tx_hash: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<i64>,
    /// Completion timestamp.
    #[serde(default)]
    pub completed_at: Option<i64>,
}

/// Type of asset operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetOperationType {
    /// Deposit from L1 or bridge.
    Deposit,
    /// Withdrawal to L1 or bridge.
    Withdrawal,
    /// Transfer between accounts.
    Transfer,
}

/// Status of asset operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetOperationStatus {
    /// Operation is pending.
    Pending,
    /// Operation is confirmed.
    Confirmed,
    /// Operation completed successfully.
    Completed,
    /// Operation failed.
    Failed,
}

/// Stark account credentials.
#[derive(Debug, Clone)]
pub struct StarkAccount {
    /// API key for authentication.
    pub api_key: String,
    /// Stark public key (hex).
    pub public_key: String,
    /// Stark private key (hex).
    pub private_key: String,
    /// Vault ID.
    pub vault_id: String,
}

impl StarkAccount {
    /// Create a new Stark account from credentials.
    pub fn new(
        api_key: impl Into<String>,
        public_key: impl Into<String>,
        private_key: impl Into<String>,
        vault_id: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            public_key: public_key.into(),
            private_key: private_key.into(),
            vault_id: vault_id.into(),
        }
    }
}
