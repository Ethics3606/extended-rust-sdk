//! Debug example to see raw API responses

use extended_rust_sdk::config::mainnet_config;
use reqwest::header;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("EXTENDED_API_KEY").expect("EXTENDED_API_KEY not set");

    let config = mainnet_config();

    let client = reqwest::Client::builder()
        .default_headers({
            let mut headers = header::HeaderMap::new();
            headers.insert(
                header::USER_AGENT,
                header::HeaderValue::from_static("extended-rust-sdk/0.1.0"),
            );
            headers.insert(
                header::ACCEPT,
                header::HeaderValue::from_static("application/json"),
            );
            headers
        })
        .build()?;

    // Test account info endpoint
    let url = format!("{}/api/v1/user/account/info", config.api_base_url);
    println!("Fetching: {}", url);

    let resp = client
        .get(&url)
        .header("X-Api-Key", &api_key)
        .send()
        .await?;

    println!("Status: {}", resp.status());
    let text = resp.text().await?;
    println!("Response:\n{}\n", text);

    // Test balance endpoint
    let url = format!("{}/api/v1/user/balance", config.api_base_url);
    println!("Fetching: {}", url);

    let resp = client
        .get(&url)
        .header("X-Api-Key", &api_key)
        .send()
        .await?;

    println!("Status: {}", resp.status());
    let text = resp.text().await?;
    println!("Response:\n{}\n", text);

    // Test positions endpoint
    let url = format!("{}/api/v1/user/positions", config.api_base_url);
    println!("Fetching: {}", url);

    let resp = client
        .get(&url)
        .header("X-Api-Key", &api_key)
        .send()
        .await?;

    println!("Status: {}", resp.status());
    let text = resp.text().await?;
    println!("Response:\n{}\n", text);

    // Test leverage endpoint
    let url = format!("{}/api/v1/user/leverage", config.api_base_url);
    println!("Fetching: {}", url);

    let resp = client
        .get(&url)
        .header("X-Api-Key", &api_key)
        .send()
        .await?;

    println!("Status: {}", resp.status());
    let text = resp.text().await?;
    println!("Response:\n{}\n", text);

    // Test fees endpoint
    let url = format!("{}/api/v1/user/fees", config.api_base_url);
    println!("Fetching: {}", url);

    let resp = client
        .get(&url)
        .header("X-Api-Key", &api_key)
        .send()
        .await?;

    println!("Status: {}", resp.status());
    let text = resp.text().await?;
    println!("Response:\n{}\n", text);

    // Try potential spot/collateral endpoints
    let endpoints = [
        "user/spot",
        "user/spot/balances",
        "user/collateral",
        "user/assets",
        "user/portfolio",
        "portfolio/spot",
        "portfolio/collateral",
        "portfolio/assets",
    ];

    for endpoint in endpoints {
        let url = format!("{}/api/v1/{}", config.api_base_url, endpoint);
        println!("Trying: {}", url);

        let resp = client
            .get(&url)
            .header("X-Api-Key", &api_key)
            .send()
            .await?;

        println!("Status: {}", resp.status());
        if resp.status().is_success() {
            let text = resp.text().await?;
            println!("Response:\n{}\n", text);
        } else {
            println!();
        }
    }

    Ok(())
}
