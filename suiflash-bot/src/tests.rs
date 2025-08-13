#[cfg(test)]
mod tests {
    use tokio;

    use super::*;
    use crate::{
        collectors::ProtocolDataCollector,
        config::{Config, FlashLoanRequest, Protocol, RouteMode},
        executors::FlashLoanExecutor,
        strategies::{ExecutionPlan, FlashLoanStrategy},
    };

    fn create_test_config() -> Config {
        Config {
            sui_rpc_url: "https://test.sui.io".to_string(),
            private_key: "test_key".to_string(),
            sui_flash_package_id: "0x1".to_string(),
            sui_flash_config_object_id: "0x2".to_string(),
            server_port: 3000,
            refresh_interval_ms: 1000,
            strategy: "cheapest".to_string(),
            contract_package_id: "0x1".to_string(),
            navi_package_id: "0x2".to_string(),
            bucket_package_id: "0x3".to_string(),
            scallop_package_id: "0x4".to_string(),
        }
    }

    fn create_test_request() -> FlashLoanRequest {
        FlashLoanRequest {
            asset: "SUI".to_string(),
            amount: 100_000, // 0.0001 SUI - smaller amount for tests
            route_mode: RouteMode::BestCost,
            explicit_protocol: None,
            user_operation: "test_operation".to_string(),
        }
    }

    #[tokio::test]
    async fn test_protocol_data_collector_creation() {
        let config = create_test_config();
        let collector = ProtocolDataCollector::new(config);

        // Test that we can get data (should be empty initially)
        let data = collector.get_all_protocol_data().await;
        assert!(data.is_empty());
    }

    #[tokio::test]
    async fn test_protocol_data_collection() {
        let config = create_test_config();
        let collector = ProtocolDataCollector::new(config);

        // Simulate data collection
        collector.collect_all_data().await.unwrap();

        // Check that data was collected for all protocols
        let data = collector.get_all_protocol_data().await;
        assert_eq!(data.len(), 3);
        assert!(data.contains_key(&Protocol::Navi));
        assert!(data.contains_key(&Protocol::Bucket));
        assert!(data.contains_key(&Protocol::Scallop));
    }

    #[tokio::test]
    async fn test_cheapest_protocol_selection() {
        let config = create_test_config();
        let collector = ProtocolDataCollector::new(config.clone());
        let strategy = FlashLoanStrategy::new(config, collector.clone());

        // Populate collector with test data
        collector.collect_all_data().await.unwrap();

        let request = create_test_request();
        let best_protocol = strategy.find_best_protocol(&request).await.unwrap();

        // Bucket should be cheapest according to our test data (5 bps)
        assert_eq!(best_protocol, Protocol::Bucket);
    }

    #[tokio::test]
    async fn test_highest_liquidity_protocol_selection() {
        let mut config = create_test_config();
        config.strategy = "highest_liquidity".to_string();

        let collector = ProtocolDataCollector::new(config.clone());
        let strategy = FlashLoanStrategy::new(config, collector.clone());

        // Populate collector with test data
        collector.collect_all_data().await.unwrap();

        let request = create_test_request();
        let best_protocol = strategy.find_best_protocol(&request).await.unwrap();

        // Navi should have highest liquidity according to our test data (10M)
        assert_eq!(best_protocol, Protocol::Navi);
    }

    #[tokio::test]
    async fn test_insufficient_liquidity() {
        let config = create_test_config();
        let collector = ProtocolDataCollector::new(config.clone());
        let strategy = FlashLoanStrategy::new(config, collector.clone());

        // Populate collector with test data
        collector.collect_all_data().await.unwrap();

        let mut request = create_test_request();
        request.amount = 20_000_000_000; // 20 SUI - exceeds all protocol liquidity

        let result = strategy.find_best_protocol(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cost_calculation() {
        let config = create_test_config();
        let collector = ProtocolDataCollector::new(config.clone());
        let strategy = FlashLoanStrategy::new(config, collector.clone());

        // Populate collector with test data
        collector.collect_all_data().await.unwrap();

        let request = create_test_request();

        // Test cost calculation for Navi (8 bps fee)
        let cost = strategy
            .calculate_cost(&request, Protocol::Navi)
            .await
            .unwrap();
        let expected_fee = (100_000u128 * 8) / 10000; // 80
        let expected_total = 100_000 + expected_fee as u64;
        assert_eq!(cost, expected_total);
    }

    #[tokio::test]
    async fn test_execution_plan_generation() {
        let config = create_test_config();
        let collector = ProtocolDataCollector::new(config.clone());
        let strategy = FlashLoanStrategy::new(config, collector.clone());

        // Populate collector with test data
        collector.collect_all_data().await.unwrap();

        let request = create_test_request();
        let plan = strategy.generate_execution_plan(&request).await.unwrap();

        assert_eq!(plan.amount, request.amount);
        assert_eq!(plan.protocol, Protocol::Bucket); // Should choose cheapest
        assert!(plan.total_cost > plan.amount); // Should include fees
    }

    #[tokio::test]
    async fn test_flash_loan_executor_creation() {
        let config = create_test_config();
        let _executor = FlashLoanExecutor::new(config);

        // Test executor creation succeeds
        // In a real implementation, this would test SUI client connectivity
        assert!(true); // Placeholder assertion
    }

    #[tokio::test]
    async fn test_flash_loan_execution() {
        let config = create_test_config();
        let executor = FlashLoanExecutor::new(config);

        let plan = ExecutionPlan {
            protocol: Protocol::Navi,
            amount: 100_000,
            total_cost: 100_080,
            user_operation: "test_operation".to_string(),
        };

        // Test execution (currently returns mock data)
        let result = executor.execute_flash_loan(&plan).await;
        assert!(result.is_ok());

        let tx_digest = result.unwrap();
        assert!(!tx_digest.is_empty());
    }

    #[tokio::test]
    async fn test_execution_verification() {
        let config = create_test_config();
        let executor = FlashLoanExecutor::new(config);

        let tx_digest = "0x1234567890abcdef";
        let result = executor.verify_execution(tx_digest).await;

        assert!(result.is_ok());
        assert!(result.unwrap()); // Mock implementation returns true
    }
}
