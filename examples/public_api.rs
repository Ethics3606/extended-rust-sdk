//! Example: Fetching public market data from Extended Exchange.
//!
//! This example demonstrates how to use the public API to fetch:
//! - Available markets
//! - Market statistics
//! - Order book data
//! - Recent trades
//! - Candlestick data
//!
//! Run with: `cargo run --example public_api`

use extended_rust_sdk::{
    config::testnet_config,
    models::{CandleType, GetCandlesParams, TimeInterval},
    TradingClient,
};

#[tokio::main]
async fn main() -> extended_rust_sdk::error::Result<()> {
    // Create a public-only client (no authentication needed)
    let client = TradingClient::public_only(testnet_config())?;
    let api = client.api();

    println!("=== Extended Exchange Public API Demo ===\n");

    // 1. Fetch all available markets
    println!("📊 Fetching markets...");
    let markets = api.get_markets().await?;
    println!("Found {} markets:", markets.len());
    for (name, market) in markets.iter().take(5) {
        println!(
            "  - {} ({}/{}): {:?}",
            name, market.base_asset(), market.quote_asset(), market.status
        );
    }
    if markets.len() > 5 {
        println!("  ... and {} more", markets.len() - 5);
    }
    println!();

    // Use BTC-USD for the following examples (or first available market)
    let market_name = if markets.contains_key("BTC-USD") {
        "BTC-USD"
    } else {
        markets.keys().next().map(|s| s.as_str()).unwrap_or("BTC-USD")
    };

    // 2. Fetch market statistics
    println!("📈 Fetching {} stats...", market_name);
    match api.get_market_stats(market_name).await {
        Ok(stats) => {
            println!("  Mark Price: {}", stats.mark_price);
            println!("  Index Price: {}", stats.index_price);
            if let Some(high) = stats.daily_high {
                println!("  24h High: {}", high);
            }
            if let Some(low) = stats.daily_low {
                println!("  24h Low: {}", low);
            }
            if let Some(vol) = stats.daily_volume {
                println!("  24h Volume: {} USD", vol);
            }
            if let Some(change) = stats.daily_price_change_percentage {
                println!("  24h Change: {}%", change);
            }
            if let Some(oi) = stats.open_interest {
                println!("  Open Interest: {}", oi);
            }
            if let Some(rate) = stats.funding_rate {
                println!("  Funding Rate: {}%", rate);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 3. Fetch order book
    println!("📖 Fetching {} orderbook (top 5 levels)...", market_name);
    match api.get_orderbook(market_name, Some(5)).await {
        Ok(orderbook) => {
            println!("  Bids:");
            for (i, bid) in orderbook.bids.iter().enumerate() {
                println!("    {}. {} @ {}", i + 1, bid.quantity, bid.price);
            }
            println!("  Asks:");
            for (i, ask) in orderbook.asks.iter().enumerate() {
                println!("    {}. {} @ {}", i + 1, ask.quantity, ask.price);
            }
            if let Some(spread) = orderbook.spread() {
                println!("  Spread: {}", spread);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 4. Fetch recent trades
    println!("💱 Fetching recent {} trades...", market_name);
    match api.get_trades(market_name, None).await {
        Ok(trades) => {
            println!("  Last {} trades:", trades.len().min(5));
            for trade in trades.iter().take(5) {
                println!(
                    "    {:?} {} @ {} (ID: {})",
                    trade.side, trade.quantity, trade.price, trade.id
                );
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 5. Fetch candles
    println!("🕯️ Fetching {} 1-hour candles...", market_name);
    let candle_params = GetCandlesParams::new(TimeInterval::OneHour).with_limit(5);
    match api.get_candles(market_name, CandleType::Trades, candle_params).await {
        Ok(candles) => {
            println!("  Last {} candles:", candles.len());
            for candle in candles.iter() {
                let trend = if candle.is_bullish() { "🟢" } else { "🔴" };
                println!(
                    "    {} O:{} H:{} L:{} C:{} V:{}",
                    trend, candle.open, candle.high, candle.low, candle.close, candle.volume
                );
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 6. Fetch funding rates
    println!("💰 Fetching {} funding rates...", market_name);
    match api.get_funding_rates(market_name, Some(5)).await {
        Ok(rates) => {
            println!("  Recent funding rates:");
            for rate in rates.iter() {
                println!("    {} at timestamp {}", rate.funding_rate, rate.funding_time);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
