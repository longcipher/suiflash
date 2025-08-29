use std::{collections::HashMap, sync::Arc};

use eyre::Result;
use serde_json::Value;
use sui_sdk::{SuiClient, SuiClientBuilder};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::Config;

#[derive(Clone)]
pub struct NaviCollector {
    #[allow(dead_code)]
    config: Config,
    #[allow(dead_code)]
    sui_client: SuiClient,
    http_client: reqwest::Client,
    fee_cache: Arc<RwLock<HashMap<String, u64>>>,
}

impl NaviCollector {
    pub async fn new(config: Config) -> Self {
        let sui_client = SuiClientBuilder::default()
            .build(&config.sui_rpc_url)
            .await
            .expect("Failed to create SUI client");

        Self {
            config,
            sui_client,
            fee_cache: Arc::new(RwLock::new(HashMap::new())),
            http_client: reqwest::Client::new(),
        }
    }

    /// Get cached flash loan fee for specific coin type
    pub async fn get_cached_fee(&self, coin_type: &str) -> Option<u64> {
        self.fee_cache.read().await.get(coin_type).copied()
    }

    /// Update all flash loan fees from Navi API
    /// Based on: https://github.com/naviprotocol/navi-sdk/blob/main/src/libs/PTB/migrate.ts#L22
    pub async fn update_all_fees(&self) -> Result<()> {
        const NAVI_FLASHLOAN_API: &str = "https://open-api.naviprotocol.io/api/navi/flashloan";

        debug!(
            "Fetching flash loan fees from Navi API: {}",
            NAVI_FLASHLOAN_API
        );

        let response = self.http_client
            .get(NAVI_FLASHLOAN_API)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "application/json")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Cache-Control", "no-cache")
            .send()
            .await
            .map_err(|e| eyre::eyre!("Failed to fetch flash loan fees: {}", e))?;

        if !response.status().is_success() {
            return Err(eyre::eyre!(
                "Failed to fetch flash loan fees: HTTP {}",
                response.status()
            ));
        }

        let fee_data: Value = response.json().await?;

        // Parse the response structure similar to TypeScript implementation
        if let Some(data) = fee_data.get("data") {
            let mut cache = self.fee_cache.write().await;
            cache.clear();

            // Process each coin's flash loan fee
            if let Some(data_obj) = data.as_object() {
                for (coin_address, coin_data) in data_obj {
                    if let Some(flashloan_fee) = coin_data.get("flashloanFee") {
                        let fee_rate = match flashloan_fee {
                            Value::Number(n) => n.as_f64().unwrap_or(0.008), // Default 0.8%
                            Value::String(s) => s.parse::<f64>().unwrap_or(0.008),
                            _ => 0.008,
                        };

                        // Convert fee rate to basis points (0.008 -> 80 bps)
                        let fee_bps = (fee_rate * 10000.0) as u64;

                        cache.insert(coin_address.clone(), fee_bps);

                        info!(
                            "Updated flash loan fee for {}: {} bps ({}%)",
                            coin_address,
                            fee_bps,
                            fee_bps as f64 / 100.0
                        );
                    }
                }
            }

            info!(
                "Successfully updated {} flash loan fees from Navi API",
                cache.len()
            );
        } else {
            warn!("No data field found in Navi API response");
        }

        Ok(())
    }

    /// Get flash loan fee for a specific coin type
    /// If not cached, will fetch from API first
    #[allow(dead_code)]
    pub async fn get_flash_loan_fee(&self, coin_type: &str) -> Result<u64> {
        // Try cached value first
        if let Some(cached_fee) = self.get_cached_fee(coin_type).await {
            debug!(
                "Using cached Navi flash loan fee for {}: {} bps",
                coin_type, cached_fee
            );
            return Ok(cached_fee);
        }

        // Update fees from API
        self.update_all_fees().await?;

        // Try cached value again
        if let Some(cached_fee) = self.get_cached_fee(coin_type).await {
            Ok(cached_fee)
        } else {
            // Default fallback if coin not found
            warn!(
                "Flash loan fee not found for coin type: {}, using default 80 bps",
                coin_type
            );
            Ok(80) // Default 0.8% = 80 bps
        }
    }

    /// Test method to print all available flash loan fees
    pub async fn print_all_fees(&self) -> Result<()> {
        self.update_all_fees().await?;

        let cache = self.fee_cache.read().await;

        println!("\n=== Navi Protocol Flash Loan Fees ===");
        for (coin_address, fee_bps) in cache.iter() {
            let fee_percentage = *fee_bps as f64 / 10000.0;
            println!("  {coin_address}: {fee_bps} bps ({fee_percentage}%)");
        }
        println!("Total coins: {}\n", cache.len());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            sui_rpc_url: "https://fullnode.testnet.sui.io:443".to_string(),
            private_key: "test_key".to_string(),
            sui_flash_package_id:
                "0xa99c85b0b24c6b2c6b88acb6ae19b2e2e4c8c11e6f9b6e9c0b0b6b9e0b0b0b0b".to_string(),
            sui_flash_config_object_id:
                "0xa99c85b0b24c6b2c6b88acb6ae19b2e2e4c8c11e6f9b6e9c0b0b6b9e0b0b0b0b".to_string(),
            server_port: 3000,
            refresh_interval_ms: 5000,
            strategy: "cheapest".to_string(),
            contract_package_id:
                "0xa99c85b0b24c6b2c6b88acb6ae19b2e2e4c8c11e6f9b6e9c0b0b6b9e0b0b0b0b".to_string(),
            navi_package_id: "0xa99c85b0b24c6b2c6b88acb6ae19b2e2e4c8c11e6f9b6e9c0b0b6b9e0b0b0b0b"
                .to_string(),
            bucket_package_id: "0xb99c85b0b24c6b2c6b88acb6ae19b2e2e4c8c11e6f9b6e9c0b0b6b9e0b0b0b0b"
                .to_string(),
            scallop_package_id:
                "0xc99c85b0b24c6b2c6b88acb6ae19b2e2e4c8c11e6f9b6e9c0b0b6b9e0b0b0b0b".to_string(),
            service_fee_bps: 1,
        }
    }

    #[tokio::test]
    async fn test_navi_collector_initialization() {
        let config = create_test_config();
        let collector = NaviCollector::new(config).await;

        // Initially cache should be empty
        assert!(collector.get_cached_fee("0x2::sui::SUI").await.is_none());
    }

    #[tokio::test]
    async fn test_navi_fee_cache_update() {
        let config = create_test_config();
        let collector = NaviCollector::new(config).await;

        // Mock cache update
        {
            let mut cache = collector.fee_cache.write().await;
            cache.insert("0x2::sui::SUI".to_string(), 100);
        }

        // Should return cached value
        assert_eq!(collector.get_cached_fee("0x2::sui::SUI").await, Some(100));
    }

    #[tokio::test]
    async fn test_navi_flash_loan_fee_real_api() {
        let config = create_test_config();
        let collector = NaviCollector::new(config).await;

        // This will call the real Navi API
        let result = collector.get_flash_loan_fee("0x2::sui::SUI").await;

        match result {
            Ok(fee) => {
                println!("Real SUI flash loan fee: {} bps", fee);
                assert!(fee > 0 && fee < 1000); // Reasonable range check
            }
            Err(e) => {
                println!(
                    "API call failed (expected in some test environments): {}",
                    e
                );
                // Don't fail the test if API is unavailable
            }
        }
    }

    #[tokio::test]
    async fn test_navi_update_all_fees() {
        let config = create_test_config();
        let collector = NaviCollector::new(config).await;

        // Test updating all fees
        let result = collector.update_all_fees().await;

        match result {
            Ok(_) => {
                let cache = collector.fee_cache.read().await;
                println!("Updated {} coin fees", cache.len());

                // Check if SUI fee is available
                if let Some(sui_fee) = cache.get(
                    "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI",
                ) {
                    println!("SUI fee: {} bps", sui_fee);
                    assert!(*sui_fee > 0);
                }
            }
            Err(e) => {
                println!(
                    "API call failed (expected in some test environments): {}",
                    e
                );
            }
        }
    }
}
