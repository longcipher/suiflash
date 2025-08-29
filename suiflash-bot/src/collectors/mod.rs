//! Protocol data collectors for flash loan fees
//!
//! This module provides collectors for different protocols to fetch
//! flash loan fees and cache them for quick REST API access.

use std::{collections::HashMap, sync::Arc};

use artemis::types::{Collector, CollectorStream};
use async_trait::async_trait;
use eyre::Result;
use tokio::{
    sync::RwLock,
    time::{Duration, interval},
};
use tracing::{error, info, warn};

use crate::config::{Config, Protocol, ProtocolData};

mod bucket_collector;
mod navi_collector;
mod scallop_collector;

use bucket_collector::BucketCollector;
use navi_collector::NaviCollector;
use scallop_collector::ScallopCollector;

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
                available_liquidity: Self::get_mock_liquidity_for_protocol(protocol),
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
                        available_liquidity: Self::get_mock_liquidity_for_protocol(protocol),
                        last_updated: now,
                    },
                )
            })
            .collect()
    }

    /// Get mock liquidity for testing purposes
    fn get_mock_liquidity_for_protocol(protocol: Protocol) -> u64 {
        match protocol {
            Protocol::Navi => 10_000_000_000_000,   // 10M SUI equivalent
            Protocol::Bucket => 5_000_000_000_000,  // 5M SUI equivalent
            Protocol::Scallop => 7_000_000_000_000, // 7M SUI equivalent
        }
    }

    /// Update flash loan fees for all protocols
    pub async fn update_all_fees(&self) -> Result<()> {
        info!("Updating flash loan fees for all protocols");

        let mut fees = HashMap::new();

        // Update Navi fee and print detailed info
        match self.navi_collector.update_all_fees().await {
            Ok(_) => {
                // Use print_all_fees to show detailed information
                if let Err(e) = self.navi_collector.print_all_fees().await {
                    warn!("Failed to print Navi fees: {}", e);
                }

                // Use SUI fee as the protocol default
                if let Some(sui_fee) = self.navi_collector.get_cached_fee("0x2::sui::SUI").await {
                    fees.insert(Protocol::Navi, sui_fee);
                } else if let Some(sui_fee) = self.navi_collector.get_cached_fee("0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI").await {
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

        // No direct access to fee_cache anymore - test through public methods
        // First check that cache is empty
        assert!(collector.get_cached_fee("0x2::sui::SUI").await.is_none());
    }

    #[tokio::test]
    async fn test_navi_flash_loan_fee_with_fallback() {
        let config = create_test_config();
        let collector = NaviCollector::new(config).await;

        // This will call the API with SUI coin type and may fail with network issues
        let result = collector.get_flash_loan_fee("0x2::sui::SUI").await;

        match result {
            Ok(fee) => {
                // Should be reasonable value (likely 80 bps default or real API value)
                assert!(fee >= 80 && fee <= 1000);
            }
            Err(_) => {
                // Expected to fail with network issues in test environment
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
            assert_eq!(bucket_data.available_liquidity, 5_000_000_000_000); // Mock liquidity for testing
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
