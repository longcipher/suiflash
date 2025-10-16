# Production Readiness Checklist - Navi Protocol Integration

## Status Overview

**Current State**: MOCK/PLACEHOLDER Implementation (~30% Complete)
**Target**: Production-Ready Navi Flash Loan Integration
**Last Updated**: 2024

---

## ✅ Completed Items

### 1. Configuration & Infrastructure
- [x] Bot default fee rate corrected (6 bps)
- [x] Config struct extended with Navi addresses
- [x] Mainnet addresses configured:
  - `navi_protocol_package`
  - `navi_storage_id`
  - `navi_flashloan_config_id`
  - `navi_incentive_v2_id`
  - `navi_incentive_v3_id`
  - `navi_price_oracle_id`
- [x] API integration for fee collection (`navi_collector.rs`)
- [x] Architecture and routing logic
- [x] Documentation structure

### 2. Documentation
- [x] `NAVI_ADDRESSES.md` - Address reference
- [x] `NAVI_INTEGRATION_REVIEW.md` - Audit report
- [x] `navi_real_example.move` - Reference implementation
- [x] Warning banners in relevant docs
- [x] README updated with integration status

---

## 🔴 Critical (P0) - Required for Production

### 1. Move Contract: Core Protocol Integration

**File**: `suiflash-contract/suiflash-router/sources/integrations/navi.move`

#### Current Issues:
```move
// WRONG - Creates zero coin instead of borrowing from Navi
public fun borrow<Asset>(
    _protocol_config: &ProtocolConfig,
    _pool_id: ID,
    amount: u64,
    _ctx: &mut TxContext
): Coin<Asset> {
    coin::zero<Asset>(ctx)  // ❌ MOCK
}

// WRONG - Destroys coin instead of repaying to Navi
public fun settle<Asset>(
    _protocol_config: &ProtocolConfig,
    _pool_id: ID,
    loan_coin: Coin<Asset>,
    _receipt: NaviReceipt<Asset>,
    _ctx: &mut TxContext
) {
    coin::destroy_zero(loan_coin);  // ❌ MOCK
    // receipt silently dropped - no repayment
}
```

#### Required Changes:

**1.1 Import Navi Protocol Module**
```move
use navi_lending::lending::{
    flash_loan_with_ctx,
    flash_repay_with_ctx
};
use navi_lending::storage::Storage;
use navi_lending::incentive_v2::IncentiveV2;
use navi_lending::pool::Pool;
```

**1.2 Implement Real `borrow()` Function**
```move
public fun borrow<Asset>(
    protocol_config: &ProtocolConfig,
    pool_id: ID,
    amount: u64,
    ctx: &mut TxContext
): (Coin<Asset>, NaviReceipt<Asset>) {
    // Get Navi storage and pool
    let storage = protocol_config.get_navi_storage();
    let pool = protocol_config.get_navi_pool<Asset>();
    
    // Call Navi's flash loan
    let (loan_coin, receipt) = flash_loan_with_ctx<Asset>(
        storage,
        pool,
        amount,
        ctx
    );
    
    (loan_coin, NaviReceipt { inner: receipt })
}
```

**1.3 Implement Real `settle()` Function**
```move
public fun settle<Asset>(
    protocol_config: &ProtocolConfig,
    pool_id: ID,
    loan_coin: Coin<Asset>,
    receipt: NaviReceipt<Asset>,
    ctx: &mut TxContext
) {
    let storage = protocol_config.get_navi_storage();
    let pool = protocol_config.get_navi_pool<Asset>();
    let NaviReceipt { inner } = receipt;
    
    // Call Navi's repayment function
    flash_repay_with_ctx<Asset>(
        storage,
        pool,
        loan_coin,
        inner,
        ctx
    );
}
```

**1.4 Fix Receipt Type**
- Current: `NaviReceipt<Asset>` is custom struct (incompatible with Navi)
- Required: Use Navi's actual receipt type or wrap it properly
- See `navi_real_example.move` for reference

**Verification Steps**:
- [ ] Code compiles with `sui move build`
- [ ] Receipt type matches Navi's `FlashLoanReceipt`
- [ ] All Navi dependencies imported correctly
- [ ] Pool ID lookup works for supported assets

---

### 2. Move Contract: Pool Configuration

**File**: `suiflash-contract/suiflash-router/sources/integrations/navi.move`

#### Required:
- [ ] Validate all pool IDs against mainnet (some current values are invalid)
- [ ] Add error handling for unsupported assets
- [ ] Implement pool ID lookup from config
- [ ] Add pool existence checks before borrow

**Reference**: `docs/NAVI_ADDRESSES.md` contains correct mainnet pool IDs

---

### 3. Rust Bot: PTB Construction

**File**: `suiflash-bot/src/executors.rs`

#### Current Issue:
PTB construction for Navi calls is incomplete or uses placeholder logic.

#### Required Implementation:

**3.1 Update `build_ptb_for_navi()`**
```rust
fn build_ptb_for_navi(
    config: &Config,
    asset: &str,
    amount: u64,
    user_operation: &UserOperation,
) -> Result<ProgrammableTransaction> {
    let mut ptb = ProgrammableTransactionBuilder::new();
    
    // 1. Get pool ID for asset
    let pool_id = get_navi_pool_id(config, asset)?;
    
    // 2. Call flash_loan_with_ctx
    let (loan_coin, receipt) = ptb.move_call(
        config.navi_protocol_package,
        "flash_loan",
        "flash_loan_with_ctx",
        vec![asset_type],
        vec![
            Argument::Input(ptb.obj(ObjectArg::SharedObject {
                id: config.navi_storage_id,
                initial_shared_version: /* lookup required */,
                mutable: true,
            })?),
            Argument::Input(ptb.obj(ObjectArg::SharedObject {
                id: pool_id,
                initial_shared_version: /* lookup required */,
                mutable: true,
            })?),
            Argument::Input(ptb.pure(amount)?),
        ],
    )?;
    
    // 3. User operation (arbitrage/liquidation)
    let result_coin = insert_user_operation(&mut ptb, loan_coin, user_operation)?;
    
    // 4. Call flash_repay_with_ctx
    ptb.move_call(
        config.navi_protocol_package,
        "flash_loan",
        "flash_repay_with_ctx",
        vec![asset_type],
        vec![
            Argument::Input(ptb.obj(ObjectArg::SharedObject {
                id: config.navi_storage_id,
                initial_shared_version: /* same as above */,
                mutable: true,
            })?),
            Argument::Input(ptb.obj(ObjectArg::SharedObject {
                id: pool_id,
                initial_shared_version: /* same as above */,
                mutable: true,
            })?),
            result_coin,
            receipt,
        ],
    )?;
    
    Ok(ptb.finish())
}
```

**3.2 Helper Functions Needed**
```rust
fn get_navi_pool_id(config: &Config, asset: &str) -> Result<ObjectID> {
    match asset {
        "SUI" => Ok(ObjectID::from_hex_literal(NAVI_SUI_POOL)?),
        "USDT" => Ok(ObjectID::from_hex_literal(NAVI_USDT_POOL)?),
        "USDC" => Ok(ObjectID::from_hex_literal(NAVI_USDC_POOL)?),
        // ... other assets
        _ => bail!("Unsupported asset for Navi: {}", asset),
    }
}

async fn get_shared_object_version(
    client: &SuiClient,
    object_id: ObjectID,
) -> Result<SequenceNumber> {
    // Query chain for initial_shared_version
    // Required for SharedObject argument construction
}
```

**Verification Steps**:
- [ ] PTB builds successfully
- [ ] Correct Move call paths (package::module::function)
- [ ] All object arguments have correct `initial_shared_version`
- [ ] Type arguments match asset types exactly
- [ ] Receipt properly threaded from borrow to repay

---

### 4. Integration Testing

**Required Tests**:

**4.1 Move Contract Tests**
```move
#[test]
fun test_navi_flash_loan_borrow_repay() {
    // Setup test environment
    let ctx = &mut tx_context::dummy();
    
    // Borrow from Navi
    let (loan, receipt) = navi::borrow<SUI>(
        &protocol_config,
        pool_id,
        1000000000, // 1 SUI
        ctx
    );
    
    // Verify loan amount
    assert!(coin::value(&loan) == 1000000000, 0);
    
    // Repay
    navi::settle<SUI>(
        &protocol_config,
        pool_id,
        loan,
        receipt,
        ctx
    );
}
```

**4.2 Rust Integration Tests**
```rust
#[tokio::test]
async fn test_navi_flash_loan_execution() {
    let config = Config::load().unwrap();
    let request = FlashLoanRequest {
        asset: "SUI".to_string(),
        amount: 1_000_000_000,
        route_mode: RouteMode::Explicit,
        explicit_protocol: Some(Protocol::Navi),
        user_operation: UserOperation::Simple,
        // ...
    };
    
    let response = execute_flash_loan(&config, &request).await;
    assert!(response.is_ok());
    assert!(response.unwrap().success);
}
```

**Verification Steps**:
- [ ] Move tests pass: `sui move test`
- [ ] Rust tests pass: `cargo test --test integration_tests`
- [ ] End-to-end flow works on devnet
- [ ] Fee calculation matches on-chain result (6 bps)

---

## 🟡 High Priority (P1) - Recommended Before Launch

### 5. Error Handling & Edge Cases

**5.1 Move Contract**
- [ ] Insufficient liquidity handling
- [ ] Invalid pool ID errors
- [ ] Receipt mismatch errors
- [ ] Asset type mismatches

**5.2 Rust Bot**
- [ ] Network errors (Navi API down)
- [ ] Gas estimation failures
- [ ] Transaction simulation failures before execution
- [ ] Retry logic for transient failures

### 6. Monitoring & Observability

- [ ] Add metrics for Navi flash loan success/failure rates
- [ ] Log fee amounts for reconciliation
- [ ] Track latency of Navi API calls
- [ ] Alert on Navi protocol configuration changes

### 7. Documentation Updates

- [ ] Update `NAVI_INTEGRATION.md` after real implementation
- [ ] Add deployment guide with verification steps
- [ ] Document known limitations and workarounds
- [ ] Add troubleshooting section

---

## 🟢 Nice-to-Have (P2) - Post-Launch Improvements

### 8. Optimization

- [ ] Cache Navi pool data to reduce API calls
- [ ] Batch multiple flash loans in single PTB if possible
- [ ] Optimize gas usage in Move calls
- [ ] Implement circuit breaker for high failure rates

### 9. Advanced Features

- [ ] Support for all Navi asset types (beyond SUI/USDT/USDC)
- [ ] Dynamic fee estimation from on-chain config
- [ ] Multi-asset flash loans in single transaction
- [ ] Integration with Navi incentive rewards

---

## Pre-Production Validation Checklist

**Before deploying to mainnet:**

1. **Code Review**
   - [ ] Security review of Move contracts
   - [ ] Code review of Rust executor logic
   - [ ] Third-party audit (recommended)

2. **Testing**
   - [ ] All unit tests pass
   - [ ] Integration tests pass on devnet
   - [ ] Manual testing on testnet with real Navi pools
   - [ ] Load testing with concurrent requests

3. **Configuration**
   - [ ] All mainnet addresses verified
   - [ ] Fee calculations validated against Navi docs
   - [ ] Gas limits configured appropriately
   - [ ] Timeout values tuned

4. **Deployment**
   - [ ] Move package deployed to mainnet
   - [ ] Package ID updated in bot config
   - [ ] Smoke test on mainnet with small amounts
   - [ ] Monitoring and alerts configured

5. **Rollback Plan**
   - [ ] Document how to disable Navi routing
   - [ ] Prepare rollback procedure
   - [ ] Keep old version deployable

---

## Estimated Effort

| Category | Effort | Priority |
|----------|--------|----------|
| Move contract core integration | 2-3 days | P0 |
| Rust PTB construction | 1-2 days | P0 |
| Integration testing | 1-2 days | P0 |
| Error handling & edge cases | 1 day | P1 |
| Documentation & deployment | 1 day | P1 |
| **Total for Production Ready** | **6-9 days** | |

---

## References

- **Navi Documentation**: <https://naviprotocol.gitbook.io/navi-protocol-docs/getting-started/flash-loan>
- **Navi SDK Reference**: <https://github.com/naviprotocol/navi-sdk>
- **Internal Docs**:
  - `docs/NAVI_ADDRESSES.md`
  - `docs/NAVI_INTEGRATION_REVIEW.md`
  - `suiflash-contract/suiflash-router/sources/integrations/navi_real_example.move`

---

## Contact & Support

For questions about this checklist or implementation:
- Review audit report: `docs/NAVI_INTEGRATION_REVIEW.md`
- Check reference implementation: `navi_real_example.move`
- Navi Protocol support: <https://discord.gg/naviprotocol>
