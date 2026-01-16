//! Debug example for order signing
//!
//! This example shows all the parameters being passed to the signing function
//! to help debug signature issues.

use extended_rust_sdk::{
    config::mainnet_config,
    models::{OrderBuilder, OrderSide, StarkAccount},
    signing::StarkSigner,
    TradingClient,
};
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

    println!("=== Extended Exchange Signing Debug ===\n");

    // Create signer with derived public key first to compare
    let signer_derived = StarkSigner::from_hex(&private_key)?;
    let derived_public_key = signer_derived.public_key_hex();

    println!("Public Key Comparison:");
    println!("  From env (EXTENDED_PUBLIC_KEY): {}", public_key_env);
    println!("  Derived from private key:       {}", derived_public_key);
    let keys_match = public_key_env.to_lowercase() == derived_public_key.to_lowercase();
    println!("  Keys match: {}", keys_match);
    if !keys_match {
        println!("  WARNING: Keys don't match! Using provided public key (like Python SDK).");
    }
    println!();

    // Use the provided public key (matching Python SDK behavior)
    let signer = StarkSigner::from_hex_with_public_key(&private_key, &public_key_env)?;

    // Create trading client
    let config = mainnet_config();
    let account = StarkAccount::new(&api_key, &public_key_env, &private_key, &vault_id);
    let client = TradingClient::new(config.clone(), account)?;
    let public_api = client.public();

    // Fetch market data
    let market_name = "BTC-USD";
    println!("Fetching {} market data...", market_name);
    let markets = public_api.get_markets().await?;
    let market = markets.get(market_name).expect("Market not found");

    println!("\nMarket L2 Config:");
    println!("  Synthetic ID: {}", market.synthetic_asset_id());
    println!("  Synthetic Resolution: {}", market.synthetic_resolution());
    println!("  Collateral ID: {}", market.collateral_asset_id());
    println!("  Collateral Resolution: {}", market.collateral_resolution());
    println!();

    // Build a test order
    let trading_config = market.config();
    let raw_price = dec!(93050.2039120);
    let limit_price = trading_config.round_price_down(raw_price);
    let quantity = trading_config.round_qty_down(dec!(0.05234233));

    println!("Order Parameters:");
    println!("  Market: {}", market_name);
    println!("  Side: BUY");
    println!("  Price: {}", limit_price);
    println!("  Quantity: {}", quantity);
    println!();

    let order = OrderBuilder::limit(market_name, OrderSide::Buy, limit_price, quantity, true, false)
        .build();

    // Calculate stark amounts (same logic as sign_order)
    let synthetic_resolution = market.synthetic_resolution();
    let synthetic_amount_human = order.quantity;
    let synthetic_amount_stark = (synthetic_amount_human * rust_decimal::Decimal::from(synthetic_resolution))
        .to_i64()
        .expect("Synthetic amount overflow");

    let collateral_amount_human = order.price * order.quantity;
    let collateral_amount_stark = (collateral_amount_human * rust_decimal::Decimal::from(COLLATERAL_RESOLUTION))
        .to_i64()
        .expect("Collateral amount overflow");

    let fee_amount_human = order.fee * collateral_amount_human;
    let fee_amount_stark = (fee_amount_human * rust_decimal::Decimal::from(COLLATERAL_RESOLUTION))
        .abs()
        .to_u64()
        .expect("Fee amount overflow");

    // Adjust signs for BUY
    let final_synthetic = synthetic_amount_stark;
    let final_collateral = -collateral_amount_stark;

    // Calculate expiration with 14-day buffer
    let expiry_seconds = (order.expiry_epoch_millis / 1000) as u64;
    let expiration = expiry_seconds + (14 * 24 * 60 * 60);

    let nonce = order.nonce.to_u64().unwrap_or(0);

    println!("Stark Amounts:");
    println!("  Synthetic (human): {}", synthetic_amount_human);
    println!("  Synthetic (stark): {}", final_synthetic);
    println!("  Collateral (human): {}", collateral_amount_human);
    println!("  Collateral (stark): {}", final_collateral);
    println!("  Fee (human): {}", fee_amount_human);
    println!("  Fee (stark): {}", fee_amount_stark);
    println!();

    println!("Order Hash Parameters (passed to get_order_hash):");
    println!("  position_id: {}", vault_id);
    println!("  base_asset_id_hex: {}", market.synthetic_asset_id());
    println!("  base_amount: {}", final_synthetic);
    println!("  quote_asset_id_hex: {}", market.collateral_asset_id());
    println!("  quote_amount: {}", final_collateral);
    println!("  fee_asset_id_hex: {}", market.collateral_asset_id());
    println!("  fee_amount: {}", fee_amount_stark);
    println!("  expiration: {}", expiration);
    println!("  salt (nonce): {}", nonce);
    println!("  user_public_key_hex: {}", derived_public_key);
    println!();

    println!("Domain Parameters:");
    println!("  name: {}", config.starknet_domain.name);
    println!("  version: {}", config.starknet_domain.version);
    println!("  chain_id: {}", config.starknet_domain.chain_id);
    println!("  revision: {}", config.starknet_domain.revision);
    println!();

    // Now actually sign it
    println!("Computing order hash and signature...");
    let signed_order = extended_rust_sdk::signing::sign_order(
        order,
        &signer,
        &vault_id,
        market.synthetic_asset_id(),
        market.synthetic_resolution(),
        &config.starknet_domain,
    )?;

    println!("\nSigned Order:");
    println!("  Order ID (hash): {}", signed_order.id);
    if let Some(ref settlement) = signed_order.settlement {
        println!("  Signature r: {}", settlement.signature.r);
        println!("  Signature s: {}", settlement.signature.s);
        println!("  Stark key: {}", settlement.stark_key);
        println!("  Collateral position: {}", settlement.collateral_position);
    }

    if let Some(ref debug) = signed_order.debugging_amounts {
        println!("\nDebugging Amounts (in settlement):");
        println!("  synthetic_amount: {}", debug.synthetic_amount);
        println!("  collateral_amount: {}", debug.collateral_amount);
        println!("  fee_amount: {}", debug.fee_amount);
    }

    // Print the full JSON that would be sent
    println!("\n=== Full Order JSON ===");
    let json = serde_json::to_string_pretty(&signed_order).expect("Failed to serialize");
    println!("{}", json);

    println!("\n=== Debug Complete ===");
    Ok(())
}
