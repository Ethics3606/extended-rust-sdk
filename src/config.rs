//! Configuration for Extended Exchange API endpoints.

/// Configuration for API endpoints.
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    /// Base URL for REST API (e.g., "https://api.starknet.extended.exchange")
    pub api_base_url: String,
    /// Base URL for WebSocket streams (for future use)
    pub stream_base_url: String,
    /// API version path
    pub api_version: String,
    /// Starknet domain for typed data signing
    pub starknet_domain: StarknetDomain,
    /// Collateral asset ID for settlement (hex string)
    pub collateral_asset_id: String,
}

/// Starknet domain information for SNIP-12 typed data signing.
/// Used for computing order message hashes.
#[derive(Debug, Clone)]
pub struct StarknetDomain {
    /// Domain name (e.g., "Perpetuals")
    pub name: String,
    /// Domain version (e.g., "v0")
    pub version: String,
    /// Chain ID (e.g., "SN_MAIN" or "SN_SEPOLIA")
    pub chain_id: String,
    /// Revision number (e.g., "1")
    pub revision: String,
}

/// Legacy alias for backwards compatibility.
pub type SigningDomain = StarknetDomain;

impl EndpointConfig {
    /// Create a new endpoint configuration.
    pub fn new(
        api_base_url: impl Into<String>,
        stream_base_url: impl Into<String>,
        starknet_domain: StarknetDomain,
        collateral_asset_id: impl Into<String>,
    ) -> Self {
        Self {
            api_base_url: api_base_url.into(),
            stream_base_url: stream_base_url.into(),
            api_version: "api/v1".to_string(),
            starknet_domain,
            collateral_asset_id: collateral_asset_id.into(),
        }
    }

    /// Get the full API URL for a given path.
    pub fn api_url(&self, path: &str) -> String {
        format!("{}/{}/{}", self.api_base_url, self.api_version, path.trim_start_matches('/'))
    }

    /// Get the full stream URL for a given path.
    pub fn stream_url(&self, path: &str) -> String {
        format!("{}/{}", self.stream_base_url, path.trim_start_matches('/'))
    }

    /// Get the signing domain (alias for starknet_domain for backwards compatibility).
    pub fn signing_domain(&self) -> &StarknetDomain {
        &self.starknet_domain
    }
}

/// Create mainnet configuration.
pub fn mainnet_config() -> EndpointConfig {
    EndpointConfig::new(
        "https://api.starknet.extended.exchange",
        "wss://api.starknet.extended.exchange",
        StarknetDomain {
            name: "Perpetuals".to_string(),
            version: "v0".to_string(),
            chain_id: "SN_MAIN".to_string(),
            revision: "1".to_string(),
        },
        "0x1", // Collateral asset ID (USDC)
    )
}

/// Create testnet configuration (Sepolia).
pub fn testnet_config() -> EndpointConfig {
    EndpointConfig::new(
        "https://api.starknet.sepolia.extended.exchange",
        "wss://api.starknet.sepolia.extended.exchange",
        StarknetDomain {
            name: "Perpetuals".to_string(),
            version: "v0".to_string(),
            chain_id: "SN_SEPOLIA".to_string(),
            revision: "1".to_string(),
        },
        "0x1", // Collateral asset ID (USDC)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url() {
        let config = mainnet_config();
        assert_eq!(
            config.api_url("info/markets"),
            "https://api.starknet.extended.exchange/api/v1/info/markets"
        );
    }

    #[test]
    fn test_api_url_with_leading_slash() {
        let config = testnet_config();
        assert_eq!(
            config.api_url("/user/balance"),
            "https://api.starknet.sepolia.extended.exchange/api/v1/user/balance"
        );
    }
}
