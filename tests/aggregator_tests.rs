use sui_flash::{SuiFlashAggregator, config::AppConfig, model::{FlashLoanRequest, ProtocolKind, RouteMode}};
use sui_flash::collector::{ManualDataSource, SharedCollector};
use std::sync::Arc;

#[tokio::test]
async fn test_best_cost_plan() {
    let cfg = AppConfig::default();
    let aggregator = SuiFlashAggregator::new(cfg.clone());
    let amount = 1_000_000u128;
    let req = FlashLoanRequest { asset: "SUI".into(), amount, user_address: "0xUSER".into(), route_mode: RouteMode::BestCost, route_hint: None };
    let plan = aggregator.build_plan(&req).await.expect("plan");
    assert_eq!(plan.steps[0].protocol, ProtocolKind::Bucket); // lowest base fee (5 bps)
    assert_eq!(plan.total_fee, amount * 35 / 10_000); // 5 + 30 service
}

#[tokio::test]
async fn test_explicit_route() {
    let cfg = AppConfig::default();
    let aggregator = SuiFlashAggregator::new(cfg.clone());
    let amount = 500_000u128;
    let req = FlashLoanRequest { asset: "SUI".into(), amount, user_address: "0xUSER".into(), route_mode: RouteMode::Explicit(ProtocolKind::Scallop), route_hint: None };
    let plan = aggregator.build_plan(&req).await.expect("plan");
    assert_eq!(plan.steps[0].protocol, ProtocolKind::Scallop);
    assert_eq!(plan.total_fee, amount * 39 / 10_000); // 9 + 30 service
}

#[tokio::test]
async fn test_best_liquidity_prefers_high_liquidity_even_if_fee_higher() {
    // Bucket has lowest fee (5bps) but we will set insufficient liquidity for amount to force choosing Navi (8bps) if liquidity qualifies.
    let cfg = AppConfig::default();
    let manual = Arc::new(ManualDataSource::default());
    // Simulate liquidity: Bucket only 400k, Navi 2M, Scallop 300k
    manual.set(ProtocolKind::Bucket, "SUI", 400_000).await;
    manual.set(ProtocolKind::Navi, "SUI", 2_000_000).await;
    manual.set(ProtocolKind::Scallop, "SUI", 300_000).await;
    let collector = Arc::new(SharedCollector::new(manual));
    let aggregator = SuiFlashAggregator::with_collector(cfg.clone(), collector);
    let amount = 1_000_000u128; // exceeds Bucket & Scallop liquidity, fits Navi
    let req = FlashLoanRequest { asset: "SUI".into(), amount, user_address: "0xUSER".into(), route_mode: RouteMode::BestLiquidity, route_hint: None };
    let plan = aggregator.build_plan(&req).await.expect("plan");
    assert_eq!(plan.steps[0].protocol, ProtocolKind::Navi); // should select Navi based on liquidity
    // Total fee = protocol base (8bps) + service (30bps) = 38bps
    assert_eq!(plan.total_fee, amount * 38 / 10_000);
}
