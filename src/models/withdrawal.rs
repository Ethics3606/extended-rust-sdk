//! Withdrawal and transfer models.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Withdrawal request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalRequest {
    /// Amount to withdraw.
    pub amount: Decimal,
    /// Recipient address (Starknet address).
    pub recipient: String,
    /// Nonce for signature.
    pub nonce: u64,
    /// Expiry timestamp (Unix ms).
    pub expiry_epoch_millis: i64,
    /// Stark signature.
    pub signature: WithdrawalSignature,
}

/// Signature for withdrawal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalSignature {
    /// Signature r component.
    pub r: String,
    /// Signature s component.
    pub s: String,
}

/// Withdrawal response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Withdrawal {
    /// Withdrawal ID.
    pub id: String,
    /// Amount withdrawn.
    pub amount: Decimal,
    /// Recipient address.
    pub recipient: String,
    /// Withdrawal status.
    pub status: WithdrawalStatus,
    /// Transaction hash (when confirmed).
    pub tx_hash: Option<String>,
    /// Creation timestamp.
    pub created_at: i64,
    /// Completion timestamp.
    pub completed_at: Option<i64>,
}

/// Withdrawal status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WithdrawalStatus {
    /// Withdrawal is pending.
    Pending,
    /// Withdrawal is being processed.
    Processing,
    /// Withdrawal is confirmed on-chain.
    Confirmed,
    /// Withdrawal completed.
    Completed,
    /// Withdrawal failed.
    Failed,
}

/// Transfer request (between sub-accounts).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferRequest {
    /// Amount to transfer.
    pub amount: Decimal,
    /// Recipient account ID.
    pub recipient_account_id: String,
    /// Nonce for signature.
    pub nonce: u64,
    /// Expiry timestamp (Unix ms).
    pub expiry_epoch_millis: i64,
    /// Stark signature.
    pub signature: TransferSignature,
}

/// Signature for transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferSignature {
    /// Signature r component.
    pub r: String,
    /// Signature s component.
    pub s: String,
}

/// Transfer response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    /// Transfer ID.
    pub id: String,
    /// Amount transferred.
    pub amount: Decimal,
    /// Sender account ID.
    pub sender_account_id: String,
    /// Recipient account ID.
    pub recipient_account_id: String,
    /// Transfer status.
    pub status: TransferStatus,
    /// Creation timestamp.
    pub created_at: i64,
}

/// Transfer status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferStatus {
    /// Transfer is pending.
    Pending,
    /// Transfer completed.
    Completed,
    /// Transfer failed.
    Failed,
}

/// Bridge configuration (EVM chain support).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeConfig {
    /// Supported chains.
    pub chains: Vec<BridgeChain>,
}

/// Supported bridge chain.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeChain {
    /// Chain ID.
    pub chain_id: u64,
    /// Chain name.
    pub name: String,
    /// Minimum deposit amount.
    pub min_deposit: Decimal,
    /// Deposit fee.
    pub deposit_fee: Decimal,
    /// Whether deposits are enabled.
    pub deposits_enabled: bool,
}

/// Bridge quote request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeQuoteRequest {
    /// Source chain ID.
    pub chain_id: u64,
    /// Amount to bridge.
    pub amount: Decimal,
}

/// Bridge quote response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeQuote {
    /// Quote ID.
    pub quote_id: String,
    /// Source chain ID.
    pub chain_id: u64,
    /// Input amount.
    pub input_amount: Decimal,
    /// Output amount (after fees).
    pub output_amount: Decimal,
    /// Bridge fee.
    pub fee: Decimal,
    /// Quote expiry timestamp.
    pub expires_at: i64,
}
