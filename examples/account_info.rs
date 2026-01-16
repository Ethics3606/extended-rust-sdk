//! Example: Fetching account information from Extended Exchange.
//!
//! This example demonstrates how to use the private API to fetch:
//! - Account info
//! - Balance and equity
//! - Open positions
//! - Leverage settings
//! - Fee structure
//!
//! You need valid API credentials from Extended Exchange to run this example.
//!
//! ## Setup
//!
//! Set the following environment variables with your credentials from
//! the Extended Exchange API management panel:
//!
//! ```bash
//! export EXTENDED_API_KEY="your-api-key-here"
//! export EXTENDED_PUBLIC_KEY="0x..."  # hex string from Extended Exchange
//! export EXTENDED_PRIVATE_KEY="0x..." # hex string from Extended Exchange
//! export EXTENDED_VAULT_ID="123456"
//! ```
//!
//! Run with: `cargo run --example account_info`

use extended_rust_sdk::{config::mainnet_config, models::StarkAccount, TradingClient};
use std::env;

fn get_env_or_exit(name: &str, example: &str) -> String {
    match env::var(name) {
        Ok(val) if !val.is_empty() => val,
        _ => {
            eprintln!("Error: {} is not set or empty", name);
            eprintln!("Example: {}={}", name, example);
            eprintln!("\nSet all required environment variables:");
            eprintln!("  EXTENDED_API_KEY     - Your API key from Extended Exchange");
            eprintln!("  EXTENDED_PUBLIC_KEY  - Stark public key (hex string from Extended)");
            eprintln!("  EXTENDED_PRIVATE_KEY - Stark private key (hex string from Extended)");
            eprintln!("  EXTENDED_VAULT_ID    - Your vault ID");
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() -> extended_rust_sdk::error::Result<()> {
    // Load credentials from environment variables with helpful error messages
    let api_key = get_env_or_exit("EXTENDED_API_KEY", "abc123...");
    let public_key = get_env_or_exit(
        "EXTENDED_PUBLIC_KEY",
        "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    );
    let private_key = get_env_or_exit(
        "EXTENDED_PRIVATE_KEY",
        "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    );
    let vault_id = get_env_or_exit("EXTENDED_VAULT_ID", "123456");

    // Create Stark account
    let account = StarkAccount::new(api_key, public_key, private_key, vault_id);

    // Create trading client
    let client = TradingClient::new(mainnet_config(), account)?;
    let private_api = client.private();

    println!("=== Extended Exchange Account Info Demo ===\n");

    // 1. Fetch account info
    println!("Fetching account info...");
    match private_api.get_account_info().await {
        Ok(info) => {
            println!("  Account ID: {}", info.get_account_id());
            println!("  Description: {}", info.get_description());
            println!("  Account Index: {}", info.get_account_index());
            println!("  Status: {:?}", info.status);
            println!("  L2 Key: {}", info.get_l2_key());
            println!("  L2 Vault: {}", info.get_l2_vault());
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 2. Fetch balance
    println!("Fetching balance...");
    match private_api.get_balance().await {
        Ok(balance) => {
            println!("  Account Balance: {} USD", balance.balance);
            println!("  Equity: {} USD", balance.equity);
            println!("  Unrealized PnL: {} USD", balance.get_unrealized_pnl());
            println!("  Initial Margin: {} USD", balance.get_initial_margin());
            println!("  Maintenance Margin: {} USD", balance.get_maintenance_margin());
            println!("  Available for Trade: {} USD", balance.get_available_for_trade());
            println!(
                "  Available for Withdrawal: {} USD",
                balance.get_available_for_withdrawal()
            );
            println!(
                "  Margin Ratio: {}%",
                balance.get_margin_ratio() * rust_decimal::Decimal::from(100)
            );
            println!("  Account Leverage: {}x", balance.get_account_leverage());

            if balance.is_at_risk() {
                println!("  WARNING: Account margin ratio is high!");
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 2b. Fetch spot balances (true value breakdown)
    println!("Fetching spot/collateral breakdown...");
    match private_api.get_spot_balances().await {
        Ok(spot) => {
            for balance in spot.iter() {
                println!(
                    "  {}: {} {} @ ${} = ${} notional ({}% contrib = ${} equity)",
                    balance.asset,
                    balance.balance,
                    balance.asset,
                    balance.index_price,
                    balance.notional_value,
                    balance.contribution_factor * rust_decimal::Decimal::from(100),
                    balance.equity_contribution
                );
            }
            println!("  ---");
            println!("  Total Notional (True Value): ${}", spot.total_notional_value());
            println!("  Total Equity Contribution:   ${}", spot.total_equity_contribution());
            println!("  Total Haircut:               ${}", spot.total_haircut());
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 3. Fetch positions
    println!("Fetching open positions...");
    match private_api.get_positions(None).await {
        Ok(positions) => {
            if positions.is_empty() {
                println!("  No open positions");
            } else {
                for pos in positions {
                    let side = if pos.is_long() { "LONG" } else { "SHORT" };
                    println!("  {} {} {}:", pos.market, side, pos.size);
                    println!("    Entry: {}", pos.entry_price);
                    println!("    Mark: {}", pos.mark_price);
                    println!("    Liquidation: {:?}", pos.liquidation_price);
                    println!(
                        "    Unrealized PnL: {} ({:.2}%)",
                        pos.unrealized_pnl,
                        pos.roe()
                    );
                    println!("    Leverage: {}x", pos.leverage);
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 4. Fetch leverage settings
    println!("Fetching leverage settings...");
    match private_api.get_leverage(None).await {
        Ok(leverages) => {
            for lev in leverages.iter().take(5) {
                if let Some(max) = lev.max_leverage {
                    println!("  {}: {}x (max: {}x)", lev.market, lev.leverage, max);
                } else {
                    println!("  {}: {}x", lev.market, lev.leverage);
                }
            }
            if leverages.len() > 5 {
                println!("  ... and {} more markets", leverages.len() - 5);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 5. Fetch fee structure
    println!("Fetching fee structure...");
    match private_api.get_fees().await {
        Ok(fees) => {
            if fees.is_empty() {
                println!("  No fee data available");
            } else {
                for fee in fees.iter().take(5) {
                    println!(
                        "  {}: maker {}%, taker {}%",
                        fee.get_market(),
                        fee.get_maker_fee_rate() * rust_decimal::Decimal::from(100),
                        fee.get_taker_fee_rate() * rust_decimal::Decimal::from(100)
                    );
                }
                if fees.len() > 5 {
                    println!("  ... and {} more markets", fees.len() - 5);
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 6. Fetch open orders
    println!("Fetching open orders...");
    match private_api.get_open_orders(None).await {
        Ok(orders) => {
            if orders.is_empty() {
                println!("  No open orders");
            } else {
                for order in orders.iter().take(5) {
                    println!(
                        "  {} {:?} {} @ {} ({:?})",
                        order.market, order.side, order.quantity, order.price, order.status
                    );
                }
                if orders.len() > 5 {
                    println!("  ... and {} more orders", orders.len() - 5);
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
