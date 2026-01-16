//! Stark key derivation and signing for Extended Exchange.
//!
//! This module implements proper Starknet signing using the `rust-crypto-lib-base` library
//! which provides cryptographically correct order hashing and ECDSA signing.

use rust_crypto_lib_base::{get_order_hash, sign_message};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use starknet::core::types::Felt;
use starknet_crypto::get_public_key;

use crate::config::StarknetDomain;
use crate::error::{ExtendedError, Result};
use crate::models::{
    CreateOrderRequest, OrderSide, SettlementSignature, StarkDebuggingOrderAmounts,
    StarkSettlementModel, TransferRequest, TransferSignature, WithdrawalRequest,
    WithdrawalSignature,
};

/// Settlement resolution for collateral (USDC) - 10^6.
const COLLATERAL_RESOLUTION: i64 = 1_000_000;

/// Stark signer for creating signatures.
#[derive(Debug, Clone)]
pub struct StarkSigner {
    private_key: Felt,
    public_key: Felt,
}

impl StarkSigner {
    /// Create a new Stark signer from a private key.
    /// The public key is derived from the private key.
    pub fn new(private_key: Felt) -> Result<Self> {
        // Derive public key from private key using proper Stark curve
        let public_key = get_public_key(&private_key);
        Ok(Self {
            private_key,
            public_key,
        })
    }

    /// Create a new Stark signer with an explicit public key.
    /// Use this when you have a registered public key that should be used for signing.
    pub fn with_public_key(private_key: Felt, public_key: Felt) -> Self {
        Self {
            private_key,
            public_key,
        }
    }

    /// Create a Stark signer from hex-encoded private key.
    ///
    /// The key can be with or without "0x" prefix.
    /// The public key is derived from the private key.
    pub fn from_hex(hex_key: &str) -> Result<Self> {
        let felt = Felt::from_hex(hex_key)
            .map_err(|e| ExtendedError::Signing(format!("Invalid hex key: {:?}", e)))?;
        Self::new(felt)
    }

    /// Create a Stark signer from hex-encoded private and public keys.
    ///
    /// Use this when you have a registered public key that may differ from the derived one.
    /// This matches how the Python SDK handles signing (using the provided public key).
    pub fn from_hex_with_public_key(private_key_hex: &str, public_key_hex: &str) -> Result<Self> {
        let private_key = Felt::from_hex(private_key_hex)
            .map_err(|e| ExtendedError::Signing(format!("Invalid private key hex: {:?}", e)))?;
        let public_key = Felt::from_hex(public_key_hex)
            .map_err(|e| ExtendedError::Signing(format!("Invalid public key hex: {:?}", e)))?;
        Ok(Self::with_public_key(private_key, public_key))
    }

    /// Check if the stored public key matches the derived public key.
    /// Returns true if they match, false otherwise.
    pub fn verify_public_key(&self) -> bool {
        let derived = get_public_key(&self.private_key);
        derived == self.public_key
    }

    /// Get the derived public key (from the private key).
    /// This may differ from the stored public key if `with_public_key` was used.
    pub fn derived_public_key(&self) -> Felt {
        get_public_key(&self.private_key)
    }

    /// Get the derived public key as hex string.
    pub fn derived_public_key_hex(&self) -> String {
        format!("{:#x}", self.derived_public_key())
    }

    /// Get the public key.
    pub fn public_key(&self) -> &Felt {
        &self.public_key
    }

    /// Get the public key as hex string.
    pub fn public_key_hex(&self) -> String {
        format!("{:#x}", self.public_key)
    }

    /// Get the private key.
    pub fn private_key(&self) -> &Felt {
        &self.private_key
    }

    /// Get the private key as hex string.
    pub fn private_key_hex(&self) -> String {
        format!("{:#x}", self.private_key)
    }

    /// Sign a message hash.
    pub fn sign(&self, message_hash: &Felt) -> Result<(Felt, Felt)> {
        let signature = sign_message(message_hash, &self.private_key)
            .map_err(|e| ExtendedError::Signing(format!("Failed to sign: {}", e)))?;
        Ok((signature.r, signature.s))
    }
}

/// Parameters needed for signing an order.
#[derive(Debug, Clone)]
pub struct OrderSigningParams {
    /// Vault ID (position_id)
    pub vault_id: u32,
    /// Synthetic asset ID (base asset settlement_external_id)
    pub synthetic_asset_id: String,
    /// Synthetic asset resolution (10^precision)
    pub synthetic_resolution: i64,
    /// Collateral asset ID (quote asset settlement_external_id)
    pub collateral_asset_id: String,
    /// Starknet domain for signing
    pub domain: StarknetDomain,
}

/// Calculate Stark amounts from human-readable order values.
fn calculate_stark_amounts(
    order: &CreateOrderRequest,
    params: &OrderSigningParams,
) -> Result<(i64, i64, u64)> {
    // Calculate synthetic amount in stark units
    let synthetic_amount_human = order.quantity;
    let synthetic_amount_stark = (synthetic_amount_human * Decimal::from(params.synthetic_resolution))
        .to_i64()
        .ok_or_else(|| ExtendedError::Signing("Synthetic amount overflow".to_string()))?;

    // Calculate collateral amount in stark units (price * quantity)
    let collateral_amount_human = order.price * order.quantity;
    let collateral_amount_stark = (collateral_amount_human * Decimal::from(COLLATERAL_RESOLUTION))
        .to_i64()
        .ok_or_else(|| ExtendedError::Signing("Collateral amount overflow".to_string()))?;

    // Calculate fee amount in stark units
    // Python SDK uses ROUND_UP for fees, so we use ceil() here
    let fee_amount_human = order.fee * collateral_amount_human;
    let fee_amount_stark = (fee_amount_human * Decimal::from(COLLATERAL_RESOLUTION))
        .abs()
        .ceil()
        .to_u64()
        .ok_or_else(|| ExtendedError::Signing("Fee amount overflow".to_string()))?;

    // Adjust signs based on buy/sell
    // For BUY: synthetic is positive (receiving), collateral is negative (paying)
    // For SELL: synthetic is negative (paying), collateral is positive (receiving)
    let (final_synthetic, final_collateral) = match order.side {
        OrderSide::Buy => (synthetic_amount_stark, -collateral_amount_stark),
        OrderSide::Sell => (-synthetic_amount_stark, collateral_amount_stark),
    };

    Ok((final_synthetic, final_collateral, fee_amount_stark))
}

/// Calculate expiration timestamp with buffer (14 days from order expiry).
/// Uses ceiling division to match Python SDK's math.ceil() behavior.
fn calculate_settlement_expiration(expiry_epoch_millis: i64) -> u64 {
    // Convert to seconds with ceiling (round up like Python's math.ceil)
    // This matches: math.ceil((expire_time + 14 days).timestamp())
    let expiry_millis = expiry_epoch_millis as u64;
    let buffer_millis = 14 * 24 * 60 * 60 * 1000_u64; // 14 days in milliseconds
    let total_millis = expiry_millis + buffer_millis;
    // Ceiling division: (a + b - 1) / b
    (total_millis + 999) / 1000
}

/// Sign an order request with full parameters.
///
/// This function computes the proper Starknet order hash and creates a valid signature.
///
/// # Arguments
/// * `order` - The order request to sign
/// * `signer` - Stark signer
/// * `params` - Order signing parameters (vault_id, asset IDs, domain)
///
/// # Returns
/// The order with settlement data and ID set from the order hash
pub fn sign_order_with_params(
    mut order: CreateOrderRequest,
    signer: &StarkSigner,
    params: &OrderSigningParams,
) -> Result<CreateOrderRequest> {
    // Calculate stark amounts
    let (synthetic_amount, collateral_amount, fee_amount) = calculate_stark_amounts(&order, params)?;

    // Get nonce as u64
    let nonce = order.nonce.to_u64().unwrap_or(0);

    // Calculate expiration
    let expiration = calculate_settlement_expiration(order.expiry_epoch_millis);

    // Compute order hash using the proper Starknet message hashing
    let order_hash = get_order_hash(
        params.vault_id.to_string(),
        params.synthetic_asset_id.clone(),
        synthetic_amount.to_string(),
        params.collateral_asset_id.clone(),
        collateral_amount.to_string(),
        params.collateral_asset_id.clone(), // fee is in collateral asset
        fee_amount.to_string(),
        expiration.to_string(),
        nonce.to_string(),
        signer.public_key_hex(),
        params.domain.name.clone(),
        params.domain.version.clone(),
        params.domain.chain_id.clone(),
        params.domain.revision.clone(),
    )
    .map_err(|e| ExtendedError::Signing(format!("Failed to compute order hash: {}", e)))?;

    // Sign the hash
    let (r, s) = signer.sign(&order_hash)?;

    // Set order ID to the hash (decimal string, matching Python SDK's str(order_hash))
    // Convert Felt to decimal string via BigUint
    let hash_bytes = order_hash.to_bytes_be();
    let hash_bigint = num_bigint::BigUint::from_bytes_be(&hash_bytes);
    order.id = hash_bigint.to_string();

    // Create settlement with signature
    order.settlement = Some(StarkSettlementModel {
        signature: SettlementSignature {
            r: format!("{:#x}", r),
            s: format!("{:#x}", s),
        },
        stark_key: signer.public_key_hex(),
        collateral_position: Decimal::from(params.vault_id),
    });

    // Add debugging amounts (optional but helpful)
    order.debugging_amounts = Some(StarkDebuggingOrderAmounts {
        synthetic_amount: Decimal::from(synthetic_amount),
        collateral_amount: Decimal::from(collateral_amount),
        fee_amount: Decimal::from(fee_amount as i64),
    });

    Ok(order)
}

/// Simplified sign_order for backwards compatibility.
///
/// Note: This version uses default asset IDs. For production use with specific markets,
/// use `sign_order_with_params` with the correct asset settlement IDs from the market data.
///
/// # Arguments
/// * `order` - The order request to sign
/// * `signer` - Stark signer
/// * `vault_id` - Vault ID (collateral position ID)
/// * `synthetic_asset_id` - Synthetic asset settlement ID (from market data)
/// * `synthetic_resolution` - Synthetic asset resolution (10^precision)
/// * `domain` - Starknet domain configuration
///
/// # Returns
/// The order with settlement data attached
pub fn sign_order(
    order: CreateOrderRequest,
    signer: &StarkSigner,
    vault_id: &str,
    synthetic_asset_id: &str,
    synthetic_resolution: i64,
    domain: &StarknetDomain,
) -> Result<CreateOrderRequest> {
    let vault_id_u32: u32 = vault_id
        .parse()
        .map_err(|e| ExtendedError::Signing(format!("Invalid vault ID: {}", e)))?;

    let params = OrderSigningParams {
        vault_id: vault_id_u32,
        synthetic_asset_id: synthetic_asset_id.to_string(),
        synthetic_resolution,
        collateral_asset_id: "0x1".to_string(), // Default USDC
        domain: domain.clone(),
    };

    sign_order_with_params(order, signer, &params)
}

/// Derive a Stark private key from an Ethereum signature.
///
/// Uses the `rust-crypto-lib-base` key derivation which follows the
/// Extended Exchange key grinding process.
pub fn get_private_key_from_eth_signature(signature: &str) -> Result<Felt> {
    rust_crypto_lib_base::get_private_key_from_eth_signature(signature)
        .map_err(|e| ExtendedError::Signing(format!("Failed to derive key: {}", e)))
}

/// Sign a withdrawal request.
pub fn sign_withdrawal(
    amount: Decimal,
    recipient: &str,
    nonce: u64,
    expiry_millis: i64,
    vault_id: &str,
    collateral_asset_id: &str,
    signer: &StarkSigner,
    domain: &StarknetDomain,
) -> Result<WithdrawalRequest> {
    let vault_id_u32: u32 = vault_id
        .parse()
        .map_err(|e| ExtendedError::Signing(format!("Invalid vault ID: {}", e)))?;

    let amount_stark = (amount * Decimal::from(COLLATERAL_RESOLUTION))
        .to_u64()
        .ok_or_else(|| ExtendedError::Signing("Amount overflow".to_string()))?;

    let expiration = calculate_settlement_expiration(expiry_millis);

    let hash = rust_crypto_lib_base::get_withdrawal_hash(
        recipient.to_string(),
        vault_id_u32.to_string(),
        collateral_asset_id.to_string(),
        amount_stark.to_string(),
        expiration.to_string(),
        nonce.to_string(),
        signer.public_key_hex(),
        domain.name.clone(),
        domain.version.clone(),
        domain.chain_id.clone(),
        domain.revision.clone(),
    )
    .map_err(|e| ExtendedError::Signing(format!("Failed to compute withdrawal hash: {}", e)))?;

    let (r, s) = signer.sign(&hash)?;

    Ok(WithdrawalRequest {
        amount,
        recipient: recipient.to_string(),
        nonce,
        expiry_epoch_millis: expiry_millis,
        signature: WithdrawalSignature {
            r: format!("{:#x}", r),
            s: format!("{:#x}", s),
        },
    })
}

/// Sign a transfer request.
pub fn sign_transfer(
    amount: Decimal,
    recipient_vault_id: &str,
    sender_vault_id: &str,
    nonce: u64,
    expiry_millis: i64,
    collateral_asset_id: &str,
    signer: &StarkSigner,
    domain: &StarknetDomain,
) -> Result<TransferRequest> {
    let amount_stark = (amount * Decimal::from(COLLATERAL_RESOLUTION))
        .to_u64()
        .ok_or_else(|| ExtendedError::Signing("Amount overflow".to_string()))?;

    let expiration = calculate_settlement_expiration(expiry_millis);

    let hash = rust_crypto_lib_base::get_transfer_hash(
        recipient_vault_id.to_string(),
        sender_vault_id.to_string(),
        collateral_asset_id.to_string(),
        amount_stark.to_string(),
        expiration.to_string(),
        nonce.to_string(),
        signer.public_key_hex(),
        domain.name.clone(),
        domain.version.clone(),
        domain.chain_id.clone(),
        domain.revision.clone(),
    )
    .map_err(|e| ExtendedError::Signing(format!("Failed to compute transfer hash: {}", e)))?;

    let (r, s) = signer.sign(&hash)?;

    Ok(TransferRequest {
        amount,
        recipient_account_id: recipient_vault_id.to_string(),
        nonce,
        expiry_epoch_millis: expiry_millis,
        signature: TransferSignature {
            r: format!("{:#x}", r),
            s: format!("{:#x}", s),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stark_signer_from_hex() {
        // Use a valid Stark private key
        let hex_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let signer = StarkSigner::from_hex(hex_key).unwrap();
        assert!(!signer.public_key().eq(&Felt::ZERO));
    }

    #[test]
    fn test_get_private_key_from_eth_signature() {
        let signature = "0x9ef64d5936681edf44b4a7ad713f3bc24065d4039562af03fccf6a08d6996eab367df11439169b417b6a6d8ce81d409edb022597ce193916757c7d5d9cbf97301c";
        let result = get_private_key_from_eth_signature(signature);
        assert!(result.is_ok());
    }
}
