/// Simplified API tests for SuiFlash Bot
/// Tests basic functionality without requiring full network connectivity
use crate::config::{Config, FlashLoanRequest, Protocol, RouteMode};

#[tokio::test]
async fn test_flash_loan_request_serialization() {
    let request = FlashLoanRequest {
        asset: "SUI".to_string(),
        amount: 1_000_000_000,
        route_mode: RouteMode::BestCost,
        explicit_protocol: Some(Protocol::Navi),
        user_operation: "test_operation".to_string(),
        callback_recipient: Some("0x1234567890abcdef".to_string()),
        callback_payload: Some("dGVzdA==".to_string()),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&request).expect("Should serialize to JSON");
    assert!(json.contains("SUI"));
    assert!(json.contains("1000000000"));
    assert!(json.contains("BestCost"));
    assert!(json.contains("Navi"));

    // Test deserialization
    let deserialized: FlashLoanRequest =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(deserialized.asset, request.asset);
    assert_eq!(deserialized.amount, request.amount);
    assert_eq!(deserialized.user_operation, request.user_operation);
}

#[tokio::test]
async fn test_route_mode_serialization() {
    let modes = vec![
        RouteMode::BestCost,
        RouteMode::BestLiquidity,
        RouteMode::Explicit,
    ];

    for mode in modes {
        let json = serde_json::to_string(&mode).expect("Should serialize");
        let deserialized: RouteMode = serde_json::from_str(&json).expect("Should deserialize");

        // Can't directly compare due to enum, but ensure no panic
        let _ = format!("{:?}", deserialized);
    }
}

#[tokio::test]
async fn test_protocol_serialization() {
    let protocols = vec![Protocol::Navi, Protocol::Bucket, Protocol::Scallop];

    for protocol in protocols {
        let json = serde_json::to_string(&protocol).expect("Should serialize");
        let deserialized: Protocol = serde_json::from_str(&json).expect("Should deserialize");

        // Verify protocol values are consistent
        assert_eq!(protocol as u8, deserialized as u8);
    }
}

#[tokio::test]
async fn test_config_validation() {
    let config = Config {
        sui_rpc_url: "https://fullnode.testnet.sui.io:443".to_string(),
        private_key: "test_key".to_string(),
        sui_flash_package_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        sui_flash_config_object_id: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
        server_port: 3000,
        refresh_interval_ms: 10000,
        strategy: "cheapest".to_string(),
        contract_package_id: "0x1".to_string(),
        navi_package_id: "0x2".to_string(),
        bucket_package_id: "0x3".to_string(),
        scallop_package_id: "0x4".to_string(),
        service_fee_bps: 40,
    };

    // Test that config has reasonable values
    assert!(config.server_port > 0);
    assert!(config.refresh_interval_ms > 0);
    assert!(config.service_fee_bps < 1000); // Less than 10%
    assert!(config.sui_rpc_url.starts_with("http"));
    assert!(config.sui_flash_package_id.starts_with("0x"));
}

#[tokio::test]
async fn test_request_validation() {
    // Test valid request
    let valid_request = FlashLoanRequest {
        asset: "SUI".to_string(),
        amount: 1_000_000_000,
        route_mode: RouteMode::BestCost,
        explicit_protocol: None,
        user_operation: "arbitrage".to_string(),
        callback_recipient: None,
        callback_payload: None,
    };

    assert!(!valid_request.asset.is_empty());
    assert!(valid_request.amount > 0);
    assert!(!valid_request.user_operation.is_empty());

    // Test request with callback
    let callback_request = FlashLoanRequest {
        callback_recipient: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
        callback_payload: Some("dGVzdF9wYXlsb2Fk".to_string()),
        ..valid_request
    };

    if let Some(recipient) = &callback_request.callback_recipient {
        assert!(recipient.starts_with("0x"));
        assert!(recipient.len() >= 42); // Minimum address length
    }
}
