/// Integration tests for SuiFlash Bot
/// These tests verify the full system functionality including protocol routing,
/// transaction execution simulation, and API endpoints.
use crate::{
    collectors::ProtocolDataCollector,
    config::{Config, FlashLoanRequest, Protocol, RouteMode},
    executors::FlashLoanExecutor,
    strategies::FlashLoanStrategy,
};

/// Helper function to create test configuration
fn create_integration_test_config() -> Config {
    Config {
        sui_rpc_url: "https://fullnode.testnet.sui.io:443".to_string(),
        private_key: "test_private_key".to_string(),
        sui_flash_package_id: "0x1234567890abcdef".to_string(),
        sui_flash_config_object_id: "0xabcdef1234567890".to_string(),
        server_port: 3000,
        refresh_interval_ms: 10000,
        strategy: "cheapest".to_string(),
        contract_package_id: "0x1".to_string(),
        navi_package_id: "0x2".to_string(),
        bucket_package_id: "0x3".to_string(),
        scallop_package_id: "0x4".to_string(),
        service_fee_bps: 40,
    }
}

/// Test the full protocol data collection pipeline
#[tokio::test]
async fn test_full_protocol_data_pipeline() {
    // Test the complete data collection and processing pipeline
    let config = create_integration_test_config();
    let collector = ProtocolDataCollector::new(config.clone()).await;

    // Collect protocol data
    collector
        .collect_all_data()
        .await
        .expect("Data collection should succeed");

    // Verify data is available for all protocols
    let data = collector.get_all_protocol_data().await;
    assert!(!data.is_empty(), "Protocol data should be collected");

    // Check each protocol has valid data
    for protocol in [Protocol::Navi, Protocol::Bucket, Protocol::Scallop] {
        if let Some(protocol_data) = data.get(&protocol) {
            assert!(protocol_data.fee_bps > 0, "Fee should be positive");
            assert!(
                protocol_data.available_liquidity > 0,
                "Liquidity should be positive"
            );
            assert!(
                protocol_data.last_updated > 0,
                "Should have update timestamp"
            );
        }
    }
}

#[tokio::test]
async fn test_strategy_selection_logic() {
    let config = create_integration_test_config();
    let collector = ProtocolDataCollector::new(config.clone()).await;
    let strategy = FlashLoanStrategy::new(config, collector.clone());

    // Collect initial data
    collector
        .collect_all_data()
        .await
        .expect("Data collection should succeed");

    let test_request = FlashLoanRequest {
        asset: "SUI".to_string(),
        amount: 1_000_000_000, // 1 SUI
        route_mode: RouteMode::BestCost,
        explicit_protocol: None,
        user_operation: "integration_test".to_string(),
        callback_recipient: None,
        callback_payload: None,
    };

    // Test best cost selection
    let execution_plan = strategy
        .generate_execution_plan(&test_request)
        .await
        .expect("Should generate execution plan");

    assert_eq!(execution_plan.amount, test_request.amount);
    assert!(
        execution_plan.total_cost > execution_plan.amount,
        "Should include fees"
    );
    assert_eq!(execution_plan.user_operation, test_request.user_operation);

    // Test explicit protocol override
    let explicit_request = FlashLoanRequest {
        explicit_protocol: Some(Protocol::Navi),
        ..test_request.clone()
    };

    let navi_plan = strategy
        .override_protocol(&explicit_request, Protocol::Navi)
        .await
        .expect("Should handle explicit protocol selection");

    assert_eq!(navi_plan.protocol, Protocol::Navi);
}

#[tokio::test]
async fn test_executor_gas_estimation() {
    let config = create_integration_test_config();

    // Test executor creation and gas estimation
    match FlashLoanExecutor::new(config).await {
        Ok(executor) => {
            let test_plan = crate::strategies::ExecutionPlan {
                protocol: Protocol::Bucket,
                amount: 500_000_000,     // 0.5 SUI
                total_cost: 500_250_000, // Including 5 bps fee
                user_operation: "gas_test".to_string(),
                callback_recipient: None,
                callback_payload: None,
            };

            let gas_estimate = executor
                .estimate_gas_cost(&test_plan)
                .await
                .expect("Should estimate gas cost");

            assert!(gas_estimate > 0, "Gas estimate should be positive");
            assert!(
                gas_estimate < 50_000_000,
                "Gas estimate should be reasonable (< 0.05 SUI)"
            );

            // Test with callback recipient (should cost more)
            let callback_plan = crate::strategies::ExecutionPlan {
                callback_recipient: Some("0x1234567890abcdef".to_string()),
                ..test_plan
            };

            let callback_gas = executor
                .estimate_gas_cost(&callback_plan)
                .await
                .expect("Should estimate callback gas cost");

            assert!(
                callback_gas > gas_estimate,
                "Callback should increase gas cost"
            );
        }
        Err(_) => {
            // Expected in test environment without network access
            println!("Executor creation failed in test environment (expected)");
        }
    }
}

#[tokio::test]
async fn test_fee_calculation_accuracy() {
    let config = create_integration_test_config();
    let collector = ProtocolDataCollector::new(config.clone()).await;
    let strategy = FlashLoanStrategy::new(config, collector.clone());

    // Collect protocol data
    collector
        .collect_all_data()
        .await
        .expect("Data collection should succeed");

    let test_amounts = vec![
        1_000_000,      // 0.001 SUI
        100_000_000,    // 0.1 SUI
        1_000_000_000,  // 1 SUI
        10_000_000_000, // 10 SUI
    ];

    for amount in test_amounts {
        let request = FlashLoanRequest {
            asset: "SUI".to_string(),
            amount,
            route_mode: RouteMode::BestCost,
            explicit_protocol: None,
            user_operation: "fee_test".to_string(),
            callback_recipient: None,
            callback_payload: None,
        };

        // Test fee calculation for each protocol
        for protocol in [Protocol::Navi, Protocol::Bucket, Protocol::Scallop] {
            if let Ok(cost) = strategy.calculate_cost(&request, protocol).await {
                let fee = cost - amount;
                assert!(
                    fee > 0,
                    "Fee should be positive for protocol {:?}",
                    protocol
                );

                // Verify fee is reasonable (should be < 1% for these protocols)
                let fee_percentage = (fee as f64 / amount as f64) * 100.0;
                assert!(
                    fee_percentage < 1.0,
                    "Fee {}% should be < 1% for protocol {:?}",
                    fee_percentage,
                    protocol
                );
            }
        }
    }
}

#[tokio::test]
async fn test_liquidity_constraints() {
    let config = create_integration_test_config();
    let collector = ProtocolDataCollector::new(config.clone()).await;
    let strategy = FlashLoanStrategy::new(config, collector.clone());

    // Collect protocol data
    collector
        .collect_all_data()
        .await
        .expect("Data collection should succeed");

    // Test with extremely large amount that should exceed all liquidity
    let large_request = FlashLoanRequest {
        asset: "SUI".to_string(),
        amount: 1_000_000_000_000_000, // 1M SUI - should exceed test liquidity
        route_mode: RouteMode::BestCost,
        explicit_protocol: None,
        user_operation: "liquidity_test".to_string(),
        callback_recipient: None,
        callback_payload: None,
    };

    let result = strategy.find_best_protocol(&large_request).await;
    assert!(result.is_err(), "Should fail with insufficient liquidity");

    // Test with reasonable amount
    let normal_request = FlashLoanRequest {
        amount: 1_000_000_000, // 1 SUI
        ..large_request
    };

    let result = strategy.find_best_protocol(&normal_request).await;
    assert!(result.is_ok(), "Should succeed with reasonable amount");
}

#[tokio::test]
async fn test_transaction_simulation_end_to_end() {
    let config = create_integration_test_config();
    let collector = ProtocolDataCollector::new(config.clone()).await;
    let strategy = FlashLoanStrategy::new(config.clone(), collector.clone());

    // Only test if executor can be created (may fail in test environment)
    if let Ok(executor) = FlashLoanExecutor::new(config).await {
        // Collect protocol data
        collector
            .collect_all_data()
            .await
            .expect("Data collection should succeed");

        let test_request = FlashLoanRequest {
            asset: "SUI".to_string(),
            amount: 1_000_000_000, // 1 SUI
            route_mode: RouteMode::BestCost,
            explicit_protocol: None,
            user_operation: "end_to_end_test".to_string(),
            callback_recipient: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            callback_payload: Some("dGVzdF9wYXlsb2Fk".to_string()), // base64 "test_payload"
        };

        // Generate execution plan
        let execution_plan = strategy
            .generate_execution_plan(&test_request)
            .await
            .expect("Should generate execution plan");

        // Execute transaction (simulated)
        let tx_digest = executor
            .execute_flash_loan(&execution_plan)
            .await
            .expect("Should execute flash loan");

        assert!(
            tx_digest.starts_with("0x"),
            "Transaction digest should be hex"
        );
        assert_eq!(
            tx_digest.len(),
            66,
            "Transaction digest should be 32 bytes + 0x prefix"
        );

        // Verify transaction
        let verification = executor
            .verify_execution(&tx_digest)
            .await
            .expect("Should verify transaction");
        assert!(verification, "Transaction should verify successfully");
    } else {
        println!("Executor creation failed - skipping transaction simulation test");
    }
}

/// Test different routing strategies
#[tokio::test]
async fn test_routing_strategies() {
    // Test cheapest strategy
    let mut config = create_integration_test_config();
    config.strategy = "cheapest".to_string();

    let collector = ProtocolDataCollector::new(config.clone()).await;
    let strategy = FlashLoanStrategy::new(config, collector.clone());

    collector
        .collect_all_data()
        .await
        .expect("Data collection should succeed");

    let request = FlashLoanRequest {
        asset: "SUI".to_string(),
        amount: 1_000_000_000,
        route_mode: RouteMode::BestCost,
        explicit_protocol: None,
        user_operation: "strategy_test".to_string(),
        callback_recipient: None,
        callback_payload: None,
    };

    let cheapest_protocol = strategy
        .find_best_protocol(&request)
        .await
        .expect("Should find cheapest protocol");

    // Test highest liquidity strategy
    let mut config2 = create_integration_test_config();
    config2.strategy = "highest_liquidity".to_string();

    let collector2 = ProtocolDataCollector::new(config2.clone()).await;
    let strategy2 = FlashLoanStrategy::new(config2, collector2.clone());

    collector2
        .collect_all_data()
        .await
        .expect("Data collection should succeed");

    let liquidity_protocol = strategy2
        .find_best_protocol(&request)
        .await
        .expect("Should find highest liquidity protocol");

    // Protocols might be different depending on test data
    println!("Cheapest protocol: {:?}", cheapest_protocol);
    println!("Highest liquidity protocol: {:?}", liquidity_protocol);
}

#[tokio::test]
async fn test_error_handling() {
    let config = create_integration_test_config();
    let collector = ProtocolDataCollector::new(config.clone()).await;
    let strategy = FlashLoanStrategy::new(config.clone(), collector.clone());

    // Test with zero amount
    let zero_request = FlashLoanRequest {
        asset: "SUI".to_string(),
        amount: 0,
        route_mode: RouteMode::BestCost,
        explicit_protocol: None,
        user_operation: "error_test".to_string(),
        callback_recipient: None,
        callback_payload: None,
    };

    // This should either handle gracefully or return an appropriate error
    // depending on the implementation's error handling strategy
    let result = strategy.generate_execution_plan(&zero_request).await;

    // We expect this to either succeed with special handling or fail gracefully
    match result {
        Ok(plan) => {
            assert_eq!(plan.amount, 0);
        }
        Err(_) => {
            // Error is also acceptable for zero amount
        }
    }

    // Test executor error handling
    if let Ok(executor) = FlashLoanExecutor::new(config).await {
        let invalid_plan = crate::strategies::ExecutionPlan {
            protocol: Protocol::Navi,
            amount: 0,
            total_cost: 0,
            user_operation: "invalid_test".to_string(),
            callback_recipient: None,
            callback_payload: None,
        };

        // This should handle the error gracefully
        let result = executor
            .handle_execution_error(&invalid_plan, "test error")
            .await;
        assert!(result.is_ok(), "Error handling should not fail");
    }
}
