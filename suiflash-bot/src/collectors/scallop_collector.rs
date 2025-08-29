use eyre::Result;
use tracing::{debug, info};

use crate::config::Config;

#[derive(Debug, Clone)]
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
    async fn test_scallop_fee_consistency() {
        let config = create_test_config();
        let collector = ScallopCollector::new(config).await;

        // Should always return the same fee
        let fee1 = collector.get_flash_loan_fee().await.unwrap();
        let fee2 = collector.get_flash_loan_fee().await.unwrap();

        assert_eq!(fee1, fee2);
        assert_eq!(fee1, 9);
    }
}
