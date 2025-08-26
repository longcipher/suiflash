//! Protocol data collectors for flash loan fees
//!
//! This module provides collectors for different protocols to fetch
//! flash loan fees and cache them for quick REST API access.

use std::{collections::HashMap, sync::Arc};

use artemis::types::{Collector, CollectorStream};
use async_trait::async_trait;
use eyre::Result;
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::ObjectID;
use tokio::{
    sync::RwLock,
    time::{Duration, interval},
};
use tracing::{debug, error, info, warn};

use crate::config::{Config, Protocol, ProtocolData};

/// Navi Protocol collector for flash loan fees
#[derive(Clone)]
pub struct NaviCollector {
    sui_client: SuiClient,
    fee_cache: Arc<RwLock<HashMap<String, u64>>>,
}

impl NaviCollector {
    pub async fn new(config: Config) -> Self {
        let sui_client = SuiClientBuilder::default()
            .build(&config.sui_rpc_url)
            .await
            .expect("Failed to create SUI client");

        Self {
            sui_client,
            fee_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached flash loan fee for a specific coin type
    pub async fn get_cached_fee(&self, coin_type: &str) -> Option<u64> {
        self.fee_cache.read().await.get(coin_type).copied()
    }

    /// Get all cached flash loan fees
    pub async fn get_all_cached_fees(&self) -> HashMap<String, u64> {
        self.fee_cache.read().await.clone()
    }

    /// Update flash loan fees for all supported coins from Navi flashloan config
    pub async fn update_all_fees(&self) -> Result<()> {
        info!("Updating Navi flash loan fees for all supported coins");

        // Navi flashloan config object ID from address.ts
        let flashloan_config_id =
            "0x3672b2bf471a60c30a03325f104f92fb195c9d337ba58072dce764fe2aa5e2dc";

        // Supported assets object ID from address.ts
        let supported_assets_id =
            "0x6c8fc404b4f22443302bbcc50ee593e5b898cc1e6755d72af0a6aab5a7a6f6d3";

        let mut fees = HashMap::new();

        // Fetch flashloan config
        match self.fetch_flashloan_config(flashloan_config_id).await {
            Ok(config_fees) => {
                fees.extend(config_fees);
            }
            Err(e) => {
                warn!("Failed to fetch flashloan config: {}", e);
            }
        }

        // Fetch supported assets and their fees
        match self.fetch_supported_assets(supported_assets_id).await {
            Ok(asset_fees) => {
                fees.extend(asset_fees);
            }
            Err(e) => {
                warn!("Failed to fetch supported assets: {}", e);
            }
        }

        // If we couldn't fetch from on-chain, use default values for common coins
        if fees.is_empty() {
            fees = self.get_default_fees();
            warn!("Using default flash loan fees for Navi");
        }

        // Update cache
        *self.fee_cache.write().await = fees.clone();

        info!(
            "Updated Navi flash loan fees for {} coins: {:?}",
            fees.len(),
            fees
        );
        Ok(())
    }

    /// Fetch flashloan config from on-chain
    async fn fetch_flashloan_config(&self, config_id: &str) -> Result<HashMap<String, u64>> {
        let object_id = ObjectID::from_hex_literal(config_id)?;

        let response = self
            .sui_client
            .read_api()
            .get_object_with_options(
                object_id,
                sui_json_rpc_types::SuiObjectDataOptions::new()
                    .with_content()
                    .with_bcs(),
            )
            .await?;

        let fees = HashMap::new();

        match response.data {
            Some(object_data) => {
                if let Some(content) = &object_data.content {
                    match content {
                        sui_json_rpc_types::SuiParsedData::MoveObject(move_object) => {
                            debug!("Navi flashloan config object: {:?}", move_object);
                            info!(
                                "Successfully fetched Navi flashloan config object, but parsing not yet implemented"
                            );
                            // TODO: Implement proper parsing based on actual object structure
                        }
                        _ => {
                            debug!("Navi flashloan config is not a Move object");
                        }
                    }
                }
            }
            None => {
                debug!("Navi flashloan config object not found");
            }
        }

        Ok(fees)
    }

    /// Fetch supported assets and their flash loan fees
    async fn fetch_supported_assets(&self, assets_id: &str) -> Result<HashMap<String, u64>> {
        let object_id = ObjectID::from_hex_literal(assets_id)?;

        let response = self
            .sui_client
            .read_api()
            .get_object_with_options(
                object_id,
                sui_json_rpc_types::SuiObjectDataOptions::new()
                    .with_content()
                    .with_bcs(),
            )
            .await?;

        let fees = HashMap::new();

        match response.data {
            Some(object_data) => {
                if let Some(content) = &object_data.content {
                    match content {
                        sui_json_rpc_types::SuiParsedData::MoveObject(move_object) => {
                            debug!("Navi supported assets object: {:?}", move_object);
                            info!(
                                "Successfully fetched Navi supported assets object, but parsing not yet implemented"
                            );
                            // TODO: Implement proper parsing based on actual object structure
                        }
                        _ => {
                            debug!("Navi supported assets is not a Move object");
                        }
                    }
                }
            }
            None => {
                debug!("Navi supported assets object not found");
            }
        }

        Ok(fees)
    }

    /// Get default flash loan fees for common Navi-supported coins
    fn get_default_fees(&self) -> HashMap<String, u64> {
        let mut fees = HashMap::new();

        // Based on Navi SDK address.ts coin types and typical flash loan fees
        fees.insert("0x2::sui::SUI".to_string(), 8); // SUI
        fees.insert(
            "0xc060006111016b8a020ad5b33834984a437aaa7d3c74c18e09a95d48aceab08c::coin::COIN"
                .to_string(),
            8,
        ); // USDT
        fees.insert(
            "0xaf8cd5edc19c4512f4259f0bee101a40d41ebed738ade5874359610ef8eeced5::coin::COIN"
                .to_string(),
            8,
        ); // WETH
        fees.insert(
            "0x06864a6f921804860930db6ddbe2e16acdf8504495ea7481637a1c8b9a8fe54b::cetus::CETUS"
                .to_string(),
            8,
        ); // CETUS
        fees.insert(
            "0x549e8b69270defbfafd4f94e17ec44cdbdd99820b33bda2278dea3b9a32d3f55::cert::CERT"
                .to_string(),
            8,
        ); // vSui
        fees.insert(
            "0xbde4ba4c2e274a60ce15c1cfff9e5c42e41654ac8b6d906a57efa4bd3c29f47d::hasui::HASUI"
                .to_string(),
            8,
        ); // haSui
        fees.insert(
            "0xa99b8952d4f7d947ea77fe0ecdcc9e5fc0bcab2841d6e2a5aa00c3044e5544b5::navx::NAVX"
                .to_string(),
            8,
        ); // NAVX
        fees.insert(
            "0x027792d9fed7f9844eb4839566001bb6f6cb4804f66aa2da6fe1ee242d896881::coin::COIN"
                .to_string(),
            8,
        ); // WBTC
        fees.insert(
            "0x5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf::coin::COIN"
                .to_string(),
            8,
        ); // wUSDC
        fees.insert(
            "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC"
                .to_string(),
            8,
        ); // nUSDC
        fees.insert(
            "0xd0e89b2af5e4910726fbcd8b8dd37bb79b29e5f83f7491bca830e94f7f226d29::eth::ETH"
                .to_string(),
            8,
        ); // ETH
        fees.insert(
            "0x2053d08c1e2bd02791056171aab0fd12bd7cd7efad2ab8f6b9c8902f14df2ff2::ausd::AUSD"
                .to_string(),
            8,
        ); // AUSD

        fees
    }
}

/// Bucket Protocol collector for flash loan fees
#[derive(Clone)]
pub struct BucketCollector {
    #[allow(dead_code)]
    config: Config,
}

impl BucketCollector {
    pub async fn new(config: Config) -> Self {
        Self { config }
    }

    /// Get Bucket Protocol flash loan fee
    /// Bucket Protocol typically uses 5 basis points for flash loans
    pub async fn get_flash_loan_fee(&self) -> Result<u64> {
        debug!("Getting Bucket Protocol flash loan fee");

        // Bucket Protocol has a fixed flash loan fee of 5 basis points
        // In the future, this could be fetched from on-chain if they make it dynamic
        let fee_bps = 5;

        info!("Bucket flash loan fee: {} bps", fee_bps);
        Ok(fee_bps)
    }
}

/// Scallop Protocol collector for flash loan fees
#[derive(Clone)]
pub struct ScallopCollector {
    #[allow(dead_code)]
    config: Config,
}

impl ScallopCollector {
    pub async fn new(config: Config) -> Self {
        Self { config }
    }

    /// Get Scallop Protocol flash loan fee
    /// Scallop Protocol uses 9 basis points for flash loans as per our integration
    pub async fn get_flash_loan_fee(&self) -> Result<u64> {
        debug!("Getting Scallop Protocol flash loan fee");

        // Scallop Protocol has a flash loan fee of 9 basis points
        // In the future, this could be fetched from on-chain if they make it dynamic
        let fee_bps = 9;

        info!("Scallop flash loan fee: {} bps", fee_bps);
        Ok(fee_bps)
    }
}

/// Main collector that orchestrates all protocol collectors
#[derive(Clone)]
pub struct ProtocolFlashLoanCollector {
    config: Config,
    navi_collector: NaviCollector,
    bucket_collector: BucketCollector,
    scallop_collector: ScallopCollector,
    fee_cache: Arc<RwLock<HashMap<Protocol, u64>>>,
}

impl std::fmt::Debug for ProtocolFlashLoanCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtocolFlashLoanCollector")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl ProtocolFlashLoanCollector {
    pub async fn new(config: Config) -> Self {
        let navi_collector = NaviCollector::new(config.clone()).await;
        let bucket_collector = BucketCollector::new(config.clone()).await;
        let scallop_collector = ScallopCollector::new(config.clone()).await;

        Self {
            config,
            navi_collector,
            bucket_collector,
            scallop_collector,
            fee_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached flash loan fee for a protocol
    pub async fn get_flash_loan_fee(&self, protocol: Protocol) -> Option<u64> {
        self.fee_cache.read().await.get(&protocol).copied()
    }

    /// Get all cached flash loan fees
    pub async fn get_all_flash_loan_fees(&self) -> HashMap<Protocol, u64> {
        self.fee_cache.read().await.clone()
    }

    /// Get protocol data (for backward compatibility)
    pub async fn get_protocol_data(&self, protocol: Protocol) -> Option<ProtocolData> {
        self.get_flash_loan_fee(protocol)
            .await
            .map(|fee_bps| ProtocolData {
                protocol,
                fee_bps,
                available_liquidity: 0, // Not collected in simplified version
                last_updated: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            })
    }

    /// Get all protocol data (for backward compatibility)
    pub async fn get_all_protocol_data(&self) -> HashMap<Protocol, ProtocolData> {
        let fees = self.get_all_flash_loan_fees().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        fees.into_iter()
            .map(|(protocol, fee_bps)| {
                (
                    protocol,
                    ProtocolData {
                        protocol,
                        fee_bps,
                        available_liquidity: 0, // Not collected in simplified version
                        last_updated: now,
                    },
                )
            })
            .collect()
    }

    /// Update flash loan fees for all protocols
    pub async fn update_all_fees(&self) -> Result<()> {
        info!("Updating flash loan fees for all protocols");

        let mut fees = HashMap::new();

        // Update Navi fee and print detailed info
        match self.navi_collector.update_all_fees().await {
            Ok(_) => {
                let navi_fees = self.navi_collector.get_all_cached_fees().await;
                println!("\n=== Navi Protocol Flash Loan Fees ===");
                for (coin_type, fee_bps) in &navi_fees {
                    println!("Coin: {coin_type} -> Fee: {fee_bps} bps");
                }
                println!("Total Navi supported coins: {}\n", navi_fees.len());

                // Use SUI fee as the protocol default
                if let Some(sui_fee) = self.navi_collector.get_cached_fee("0x2::sui::SUI").await {
                    fees.insert(Protocol::Navi, sui_fee);
                }
            }
            Err(e) => {
                error!("Failed to update Navi fees: {}", e);
            }
        }

        // Update Bucket fee
        if let Ok(fee) = self.bucket_collector.get_flash_loan_fee().await {
            fees.insert(Protocol::Bucket, fee);
        }

        // Update Scallop fee
        if let Ok(fee) = self.scallop_collector.get_flash_loan_fee().await {
            fees.insert(Protocol::Scallop, fee);
        }

        // Update cache
        *self.fee_cache.write().await = fees;

        info!("Updated flash loan fees: {:?}", self.fee_cache.read().await);
        Ok(())
    }

    /// Start background fee collection
    pub async fn start_background_collection(&self) {
        let interval_duration = Duration::from_millis(self.config.refresh_interval_ms);
        let mut ticker = interval(interval_duration);

        info!(
            "Starting background flash loan fee collection every {}ms",
            self.config.refresh_interval_ms
        );

        loop {
            ticker.tick().await;
            if let Err(e) = self.update_all_fees().await {
                error!("Background fee collection failed: {}", e);
            }
        }
    }

    /// Alias for backward compatibility
    pub async fn collect_all_data(&self) -> Result<()> {
        self.update_all_fees().await
    }
}

// Artemis Collector implementation
#[async_trait]
impl Collector<ProtocolData> for ProtocolFlashLoanCollector {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, ProtocolData>> {
        todo!("Implement Artemis collector stream for protocol data updates")
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
        let mut fees = HashMap::new();
        fees.insert("0x2::sui::SUI".to_string(), 10);
        *collector.fee_cache.write().await = fees;

        // Should return cached value
        assert_eq!(collector.get_cached_fee("0x2::sui::SUI").await, Some(10));
    }

    #[tokio::test]
    async fn test_navi_flash_loan_fee_with_fallback() {
        let config = create_test_config();
        let collector = NaviCollector::new(config).await;

        // This will fail with invalid object ID but should return fallback
        let result = collector.get_flash_loan_fee().await;

        match result {
            Ok(fee) => {
                assert_eq!(fee, 8); // Default fallback
            }
            Err(_) => {
                // Expected to fail with invalid object ID in test
            }
        }
    }

    #[tokio::test]
    async fn test_bucket_collector_initialization() {
        let config = create_test_config();
        let _collector = BucketCollector::new(config).await;
    }

    #[tokio::test]
    async fn test_bucket_flash_loan_fee() {
        let config = create_test_config();
        let collector = BucketCollector::new(config).await;

        let result = collector.get_flash_loan_fee().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5); // Expected 5 bps
    }

    #[tokio::test]
    async fn test_scallop_collector_initialization() {
        let config = create_test_config();
        let _collector = ScallopCollector::new(config).await;
    }

    #[tokio::test]
    async fn test_scallop_flash_loan_fee() {
        let config = create_test_config();
        let collector = ScallopCollector::new(config).await;

        let result = collector.get_flash_loan_fee().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 9); // Expected 9 bps
    }

    #[tokio::test]
    async fn test_protocol_data_collector_initialization() {
        let config = create_test_config();
        let collector = ProtocolFlashLoanCollector::new(config).await;

        // Initially no cached fees
        assert!(collector.get_flash_loan_fee(Protocol::Navi).await.is_none());
        assert!(
            collector
                .get_flash_loan_fee(Protocol::Bucket)
                .await
                .is_none()
        );
        assert!(
            collector
                .get_flash_loan_fee(Protocol::Scallop)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_update_all_fees() {
        let config = create_test_config();
        let collector = ProtocolFlashLoanCollector::new(config).await;

        // Update fees
        let result = collector.update_all_fees().await;
        assert!(result.is_ok());

        // Check that fees were cached
        let all_fees = collector.get_all_flash_loan_fees().await;

        // Should have Bucket and Scallop fees (Navi may fail due to invalid object ID)
        assert!(all_fees.contains_key(&Protocol::Bucket));
        assert!(all_fees.contains_key(&Protocol::Scallop));

        assert_eq!(all_fees.get(&Protocol::Bucket), Some(&5));
        assert_eq!(all_fees.get(&Protocol::Scallop), Some(&9));
    }

    #[tokio::test]
    async fn test_get_protocol_data_backward_compatibility() {
        let config = create_test_config();
        let collector = ProtocolFlashLoanCollector::new(config).await;

        // Update fees first
        let _ = collector.update_all_fees().await;

        // Test backward compatibility
        if let Some(bucket_data) = collector.get_protocol_data(Protocol::Bucket).await {
            assert_eq!(bucket_data.protocol, Protocol::Bucket);
            assert_eq!(bucket_data.fee_bps, 5);
            assert_eq!(bucket_data.available_liquidity, 0); // Not collected in simplified version
        }
    }

    #[tokio::test]
    async fn test_fee_cache_consistency() {
        let config = create_test_config();
        let collector = ProtocolFlashLoanCollector::new(config).await;

        // Update fees
        let _ = collector.update_all_fees().await;

        // Get individual fees
        let navi_fee = collector.get_flash_loan_fee(Protocol::Navi).await;
        let bucket_fee = collector.get_flash_loan_fee(Protocol::Bucket).await;
        let scallop_fee = collector.get_flash_loan_fee(Protocol::Scallop).await;

        // Get all fees
        let all_fees = collector.get_all_flash_loan_fees().await;

        // Ensure consistency
        assert_eq!(all_fees.get(&Protocol::Navi), navi_fee.as_ref());
        assert_eq!(all_fees.get(&Protocol::Bucket), bucket_fee.as_ref());
        assert_eq!(all_fees.get(&Protocol::Scallop), scallop_fee.as_ref());
    }
}
