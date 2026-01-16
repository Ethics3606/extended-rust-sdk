use extended_rust_sdk::{
    config::mainnet_config,
    models::{OrderBuilder, OrderSide, StarkAccount},
    signing::{sign_order, StarkSigner},
    TradingClient,
};
use rust_decimal_macros::dec;
use std::{env, time::Duration};

#[tokio::main]
async fn main() -> extended_rust_sdk::error::Result<()> {
    // Load credentials from environment variables
    let api_key = env::var("EXTENDED_API_KEY").expect("EXTENDED_API_KEY not set");
    let public_key = env::var("EXTENDED_PUBLIC_KEY").expect("EXTENDED_PUBLIC_KEY not set");
    let private_key = env::var("EXTENDED_PRIVATE_KEY").expect("EXTENDED_PRIVATE_KEY not set");
    let vault_id = env::var("EXTENDED_VAULT_ID").expect("EXTENDED_VAULT_ID not set");

    // Create Stark account and signer
    // Use from_hex_with_public_key to use the registered public key (matching Python SDK behavior)
    let account = StarkAccount::new(&api_key, &public_key, &private_key, &vault_id);
    let signer = StarkSigner::from_hex_with_public_key(&private_key, &public_key)?;

    // Create trading client
    let config = mainnet_config();
    let client = TradingClient::new(config.clone(), account)?;
    let public_api = client.public();
    let private_api = client.private();

    println!("=== Extended Exchange Order Placement Demo ===\n");

    // 1. Fetch market config (for tick_size, step_size, and L2 config)
    let market_name = "ETH-USD";
    println!("Fetching {} market data...", market_name);
    let markets = public_api.get_markets().await?;
    let market = markets.get(market_name).expect("Market not found");
    let trading_config = market.config();

    println!("  Tick Size: {}", trading_config.tick_size());
    println!("  Step Size: {}", trading_config.step_size());
    println!("  Synthetic Asset ID: {}", market.synthetic_asset_id());
    println!("  Synthetic Resolution: {}", market.synthetic_resolution());
    println!();

    // 2. Build a limit order
    // IMPORTANT: Round price and quantity to market's tick/step size
    let raw_price = dec!(3350.2039120);
    let limit_price = trading_config.round_price_down(raw_price);
    let quantity = trading_config.round_qty_down(dec!(0.01));

    println!("Building limit order...");
    println!("  Side: BUY");
    println!("  Price: {} (rounded from {})", limit_price, raw_price);
    println!("  Quantity: {} ETH", quantity);
    println!("  Type: Limit, Post-Only");

    // Fee and nonce are auto-generated. Use .fee() or .nonce() to override.
    let order = OrderBuilder::limit(market_name, OrderSide::Buy, limit_price, quantity, false, false)
        .build();

    // 3. Sign the order with proper Stark crypto
    println!("\nSigning order with Stark key...");
    let signed_order = sign_order(
        order,
        &signer,
        &vault_id,
        market.synthetic_asset_id(),
        market.synthetic_resolution(),
        &config.starknet_domain,
    )?;
    println!("  Settlement attached: {:?}", signed_order.settlement.is_some());
    println!("  Order ID: {}", signed_order.id);

    // Print the full JSON that will be sent
    println!("\n=== Full Order JSON ===");
    let json = serde_json::to_string_pretty(&signed_order).expect("Failed to serialize");
    println!("{}", json);
    println!("=== End JSON ===\n");

    // 4. Submit the order
    println!("Submitting order...");
    match private_api.create_order(signed_order).await {
        Ok(result) => {
            println!("  Order placed successfully!");
            println!("  Order ID: {}", result.id);
            println!("  External ID: {}", result.external_id);

            tokio::time::sleep(Duration::from_secs_f64(5.0)).await;

            println!("Fetching order...");

            match private_api.get_order(&result.id).await {
                Ok(order) => {
                    println!("Order: {:?}", order);
                }
                Err(e) => println!("  get_order error: {}", e),
            }

            // Wait a bit before cancelling
            tokio::time::sleep(Duration::from_secs_f64(2.0)).await;

            // 5. Cancel the order
            println!("\nCancelling order...");
            match private_api.cancel_order(&result.id).await {
                Ok(()) => {
                    println!("  Order cancelled successfully!");
                }
                Err(e) => println!("  Cancel error: {}", e),
            }
        }
        Err(e) => println!("  Order error: {}", e),
    }

    Ok(())
}
