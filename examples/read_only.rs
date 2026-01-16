//! Example: Read-only account access using only API key.
//!
//! This example demonstrates how to fetch account data without Stark keys.
//! Perfect for portfolio trackers, dashboards, and monitoring apps.
//!
//! ## Setup
//!
//! Set only one environment variable:
//!
//! ```bash
//! export EXTENDED_API_KEY="your-api-key-here"
//! ```
//!
//! Run with: `cargo run --example read_only`

use extended_rust_sdk::{config::mainnet_config, ReadOnlyClient};
use std::env;

#[tokio::main]
async fn main() -> extended_rust_sdk::error::Result<()> {
    // Only need API key - no Stark credentials required
    let api_key = env::var("EXTENDED_API_KEY").expect("EXTENDED_API_KEY not set");

    let client = ReadOnlyClient::new(mainnet_config(), &api_key)?;

    println!("=== Extended Exchange Read-Only Demo ===\n");

    // 1. Account info
    println!("Account Info:");
    match client.private().get_account_info().await {
        Ok(info) => {
            println!("  ID: {}", info.get_account_id());
            println!("  Status: {:?}", info.status);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 2. Balance summary
    println!("Balance:");
    match client.private().get_balance().await {
        Ok(balance) => {
            println!("  Equity: ${}", balance.equity);
            println!("  Available for Trade: ${}", balance.get_available_for_trade());
            println!("  Unrealized PnL: ${}", balance.get_unrealized_pnl());
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 3. Spot balances (true value breakdown)
    println!("Collateral Breakdown:");
    match client.private().get_spot_balances().await {
        Ok(spot) => {
            for b in spot.iter() {
                let haircut_pct = if b.contribution_factor < rust_decimal::Decimal::ONE {
                    format!(" ({}% haircut)", (rust_decimal::Decimal::ONE - b.contribution_factor) * rust_decimal::Decimal::from(100))
                } else {
                    String::new()
                };
                println!(
                    "  {}: ${:.2} notional{} -> ${:.2} equity balance: {} index p: {}",
                    b.asset, b.notional_value, haircut_pct, b.equity_contribution, b.balance, b.index_price
                );
            }
            println!("  ---");
            println!("  True Total Value: ${:.2}", spot.total_notional_value());
            println!("  Equity (after haircuts): ${:.2}", spot.total_equity_contribution());
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 4. Positions
    println!("Open Positions:");
    match client.private().get_positions(None).await {
        Ok(positions) => {
            if positions.is_empty() {
                println!("  No open positions");
            } else {
                for pos in &positions {
                    let side = if pos.is_long() { "LONG" } else { "SHORT" };
                    println!(
                        "  {} {} {} @ ${} (PnL: ${})",
                        pos.market, side, pos.size, pos.entry_price, pos.unrealized_pnl
                    );
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 5. Open orders
    println!("Open Orders:");
    match client.private().get_open_orders(None).await {
        Ok(orders) => {
            if orders.is_empty() {
                println!("  No open orders");
            } else {
                for order in orders.iter().take(5) {
                    println!(
                        "  {} {:?} {} @ ${}",
                        order.market, order.side, order.quantity, order.price
                    );
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n=== Done ===");
    Ok(())
}
