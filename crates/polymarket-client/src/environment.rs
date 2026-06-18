use polymarket_types::EvmAddress;

/// Production and preproduction environment configuration.
#[derive(Clone, Debug)]
pub struct Environment {
    pub name: &'static str,
    pub chain_id: u64,
    pub rpc: &'static str,
    pub clob: &'static str,
    pub gamma: &'static str,
    pub data: &'static str,
    pub relayer: &'static str,
    pub rfq: &'static str,
    pub clob_ws: &'static str,
    pub rtds_ws: &'static str,
    pub sports_ws: &'static str,
    pub collateral_token: EvmAddress,
}

impl Environment {
    /// Polymarket production environment.
    #[must_use]
    pub fn production() -> Self {
        Self {
            name: "production",
            chain_id: 137,
            rpc: "https://polygon.drpc.org",
            clob: "https://clob.polymarket.com",
            gamma: "https://gamma-api.polymarket.com",
            data: "https://data-api.polymarket.com",
            relayer: "https://relayer-v2.polymarket.com",
            rfq: "https://combos-rfq-api.polymarket.com",
            clob_ws: "wss://ws-subscriptions-clob.polymarket.com",
            rtds_ws: "wss://ws-live-data.polymarket.com",
            sports_ws: "wss://sports-api.polymarket.com/ws",
            collateral_token: EvmAddress::from_str("0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB")
                .expect("valid production collateral token address"),
        }
    }

    /// Polymarket preproduction environment (HTTP endpoints only differ from production).
    #[must_use]
    pub fn preproduction() -> Self {
        Self {
            name: "preproduction",
            clob: "https://clob-staging.polymarket.com",
            gamma: "https://gamma-staging.polymarket.com",
            data: "https://data-staging.polymarket.com",
            relayer: "https://relayer-staging.polymarket.com",
            ..Self::production()
        }
    }
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_urls_are_https() {
        let env = Environment::production();
        assert!(env.gamma.starts_with("https://"));
        assert!(env.clob.starts_with("https://"));
    }
}
