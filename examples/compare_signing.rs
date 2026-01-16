//! Compare signing parameters between Rust and Python SDK.
//!
//! This example uses FIXED values for nonce and expiration so you can
//! compare the order hash output directly with the Python debug script.

use extended_rust_sdk::{
    config::mainnet_config,
    models::{OrderBuilder, OrderSide, StarkAccount},
    signing::StarkSigner,
    TradingClient,
};
use rust_crypto_lib_base::get_order_hash;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use std::env;

/// Settlement resolution for collateral (USDC) - 10^6.
const COLLATERAL_RESOLUTION: i64 = 1_000_000;

#[tokio::main]
async fn main() -> extended_rust_sdk::error::Result<()> {
    // Load credentials from environment variables
    let api_key = env::var("EXTENDED_API_KEY").expect("EXTENDED_API_KEY not set");
    let public_key_env = env::var("EXTENDED_PUBLIC_KEY").expect("EXTENDED_PUBLIC_KEY not set");
    let private_key = env::var("EXTENDED_PRIVATE_KEY").expect("EXTENDED_PRIVATE_KEY not set");
    let vault_id = env::var("EXTENDED_VAULT_ID").expect("EXTENDED_VAULT_ID not set");

    println!("=== Rust SDK Signing Comparison ===\n");

    // Create signer with provided public key
    let signer = StarkSigner::from_hex_with_public_key(&private_key, &public_key_env)?;

    // Also derive to compare
    let signer_derived = StarkSigner::from_hex(&private_key)?;
    let derived_public_key = signer_derived.public_key_hex();

    // Normalize keys for comparison (handle with/without 0x prefix)
    let env_key_normalized = public_key_env.trim_start_matches("0x").to_lowercase();
    let derived_normalized = derived_public_key.trim_start_matches("0x").to_lowercase();

    println!("Public Key Comparison:");
    println!("  From env:  {}", public_key_env);
    println!("  Derived:   {}", derived_public_key);
    println!("  Match: {}", env_key_normalized == derived_normalized);
    println!();

    // Create trading client to fetch market data
    let config = mainnet_config();
    let account = StarkAccount::new(&api_key, &public_key_env, &private_key, &vault_id);
    let client = TradingClient::new(config.clone(), account)?;
    let public_api = client.public();

    // Fetch market data
    let market_name = "BTC-USD";
    println!("Fetching {} market data...", market_name);
    let markets = public_api.get_markets().await?;
    let market = markets.get(market_name).expect("Market not found");

    let synthetic_asset_id = market.synthetic_asset_id();
    let synthetic_resolution = market.synthetic_resolution();
    let collateral_asset_id = market.collateral_asset_id();

    println!("\nMarket L2 Config:");
    println!("  Synthetic ID: {}", synthetic_asset_id);
    println!("  Synthetic Resolution: {}", synthetic_resolution);
    println!("  Collateral ID: {}", collateral_asset_id);
    println!("  Collateral Resolution: {}", market.collateral_resolution());
    println!();

    // FIXED test values for comparison
    let price = dec!(93050);
    let quantity = dec!(0.05234);

    println!("Order Parameters (FIXED for comparison):");
    println!("  Market: {}", market_name);
    println!("  Side: BUY");
    println!("  Price: {}", price);
    println!("  Quantity: {}", quantity);
    println!();

    // Calculate stark amounts
    let synthetic_amount_stark = (quantity * rust_decimal::Decimal::from(synthetic_resolution))
        .to_i64()
        .expect("Synthetic amount overflow");

    let collateral_amount_human = price * quantity;
    let collateral_amount_stark = (collateral_amount_human * rust_decimal::Decimal::from(COLLATERAL_RESOLUTION))
        .to_i64()
        .expect("Collateral amount overflow");

    let fee_rate = dec!(0.0005);
    let fee_amount_human = fee_rate * collateral_amount_human;
    let fee_amount_stark = (fee_amount_human * rust_decimal::Decimal::from(COLLATERAL_RESOLUTION))
        .abs()
        .to_u64()
        .expect("Fee amount overflow");

    // BUY: synthetic positive, collateral negative
    let final_synthetic = synthetic_amount_stark;
    let final_collateral = -collateral_amount_stark;

    println!("Stark Amounts:");
    println!("  Synthetic (human): {}", quantity);
    println!("  Synthetic (stark): {}", final_synthetic);
    println!("  Collateral (human): {}", collateral_amount_human);
    println!("  Collateral (stark): {}", final_collateral);
    println!("  Fee (human): {}", fee_amount_human);
    println!("  Fee (stark): {}", fee_amount_stark);
    println!();

    // FIXED values for comparison - use these same values in Python!
    let nonce: u64 = 12345678901234567890;
    let expiration: u64 = 1800000000; // Far future timestamp

    println!("Fixed Values (use same in Python):");
    println!("  Nonce: {}", nonce);
    println!("  Expiration: {}", expiration);
    println!();

    println!("Domain Parameters:");
    println!("  name: {}", config.starknet_domain.name);
    println!("  version: {}", config.starknet_domain.version);
    println!("  chain_id: {}", config.starknet_domain.chain_id);
    println!("  revision: {}", config.starknet_domain.revision);
    println!();

    let vault_id_u32: u32 = vault_id.parse().expect("Invalid vault ID");

    println!("Order Hash Parameters:");
    println!("  position_id: {}", vault_id_u32);
    println!("  base_asset_id_hex: {}", synthetic_asset_id);
    println!("  base_amount: {}", final_synthetic);
    println!("  quote_asset_id_hex: {}", collateral_asset_id);
    println!("  quote_amount: {}", final_collateral);
    println!("  fee_asset_id_hex: {}", collateral_asset_id);
    println!("  fee_amount: {}", fee_amount_stark);
    println!("  expiration: {}", expiration);
    println!("  salt: {}", nonce);
    println!("  user_public_key_hex: {}", signer.public_key_hex());
    println!();

    // Compute order hash directly
    println!("Computing order hash...");
    let order_hash = get_order_hash(
        vault_id_u32.to_string(),
        synthetic_asset_id.to_string(),
        final_synthetic.to_string(),
        collateral_asset_id.to_string(),
        final_collateral.to_string(),
        collateral_asset_id.to_string(),
        fee_amount_stark.to_string(),
        expiration.to_string(),
        nonce.to_string(),
        signer.public_key_hex(),
        config.starknet_domain.name.clone(),
        config.starknet_domain.version.clone(),
        config.starknet_domain.chain_id.clone(),
        config.starknet_domain.revision.clone(),
    )
    .map_err(|e| extended_rust_sdk::error::ExtendedError::Signing(format!("Hash error: {}", e)))?;

    println!("  Order hash: {:#x}", order_hash);
    println!();

    // Sign
    println!("Signing order hash...");
    let (r, s) = signer.sign(&order_hash)?;
    println!("  Signature r: {:#x}", r);
    println!("  Signature s: {:#x}", s);
    println!();

    println!("Settlement:");
    println!("  signature.r: {:#x}", r);
    println!("  signature.s: {:#x}", s);
    println!("  stark_key: {}", signer.public_key_hex());
    println!("  collateral_position: {}", vault_id_u32);
    println!();

    println!("=== Comparison Complete ===");
    println!();
    println!("Now run the Python script with the SAME fixed values and compare order hashes!");

    Ok(())
}
