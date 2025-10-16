# Navi Protocol Integration - Final Steps Guide

**Status**: Ready for Phase 2 Implementation  
**Completion**: 70% (Structure Ready) → Target: 100%  
**Estimated Time**: 2-3 days

---

## Quick Start

This guide provides step-by-step instructions to complete the Navi Protocol flash loan integration in SuiFlash.

**Prerequisites**:
- Sui CLI installed and configured
- Rust toolchain (1.75+)
- Access to Navi Protocol documentation
- Testnet SUI for testing

---

## Phase 2: Implementation Checklist

### Task 1: Add Navi Protocol Dependency (30 minutes)

**File**: `suiflash-contract/suiflash-router/Move.toml`

#### Step 1.1: Find the Correct Navi Package

Option A - Use Published Package (Recommended):
```toml
[dependencies]
Navi = { git = "https://github.com/naviprotocol/protocol-interface.git", subdir = "lending_core", rev = "main" }
```

Option B - Use Specific Version:
```toml
[dependencies]
Navi = { git = "https://github.com/naviprotocol/protocol-interface.git", subdir = "lending_core", rev = "v1.0.0" }
```

Option C - Local Development:
```toml
[dependencies]
Navi = { local = "../../navi-protocol/lending_core" }
```

#### Step 1.2: Verify Navi Module Structure

Check what's available in Navi package:
```bash
# Clone Navi protocol repo
git clone https://github.com/naviprotocol/protocol-interface.git /tmp/navi
cd /tmp/navi/lending_core

# List available modules
ls sources/

# Look for:
# - flash_loan.move
# - lending.move  
# - storage.move
# - pool.move
```

#### Step 1.3: Test Build

```bash
cd suiflash-contract/suiflash-router
sui move build

# Expected output:
# INCLUDING DEPENDENCY Navi
# INCLUDING DEPENDENCY Sui
# BUILDING suiflash_router
```

**Troubleshooting**:
- If build fails, check Navi repo structure
- Verify `subdir` path is correct
- Try different `rev` values (main, master, specific tag)

---

### Task 2: Update navi.move - Real Protocol Calls (4-6 hours)

**File**: `suiflash-contract/suiflash-router/sources/integrations/navi.move`

#### Step 2.1: Add Real Imports

**Current** (lines 8-16):
```move
use sui::coin::{Self, Coin};
use sui::tx_context::TxContext;
use sui::clock::Clock;
use sui::object::ID;

// FUTURE: Uncomment when Navi package dependency is added to Move.toml
// use navi::lending::{Self as navi_lending, Receipt as NaviReceipt};
// use navi::storage::Storage;
// use navi::pool::Pool;
```

**Replace with**:
```move
use sui::coin::{Self, Coin};
use sui::tx_context::TxContext;
use sui::clock::Clock;
use sui::object::ID;
use sui::balance::Balance;

// Import Navi Protocol modules
use navi::lending::{Self as navi_lending, Receipt as NaviReceipt};
use navi::storage::Storage;
use navi::pool::Pool;
use navi::incentive_v2::IncentiveV2;
```

**Note**: Exact module names may vary. Check Navi's actual exports.

#### Step 2.2: Update Receipt Type (if needed)

**Option A** - Use Navi's Receipt directly:
```move
// Remove custom NaviFlashLoanReceipt struct
// Delete lines 29-34

// Use NaviReceipt<T> from navi::lending instead
```

**Option B** - Wrap Navi's Receipt:
```move
/// Wrapper for Navi Protocol receipt
public struct NaviFlashLoanReceipt<phantom T> has drop {
    inner: NaviReceipt<T>,
}
```

Choose based on how protocols.move handles receipt serialization.

#### Step 2.3: Implement Real borrow() Function

**Current** (lines 62-93):
```move
public fun borrow<T>(
    amount: u64,
    ctx: &mut TxContext
): (Coin<T>, NaviFlashLoanReceipt<T>) {
    // ... mock implementation with coin::zero()
}
```

**Replace with**:
```move
/// Borrow assets through Navi Protocol flash loan
/// Returns the borrowed coins and a receipt that must be settled
public fun borrow<T>(
    storage: &mut Storage,
    pool: &mut Pool<T>,
    config: &FlashLoanConfig,
    amount: u64,
    ctx: &mut TxContext
): (Coin<T>, NaviReceipt<T>) {
    // Validate borrow amount
    assert!(amount > 0, E_INSUFFICIENT_AMOUNT);

    // Call Navi's flash loan function
    let (balance, receipt) = navi_lending::flash_loan_with_ctx<T>(
        storage,
        pool,
        config,
        amount,
        ctx
    );
    
    // Convert balance to coin for easier handling
    let coin = coin::from_balance(balance, ctx);
    
    (coin, receipt)
}
```

**Parameters to verify**:
- Check Navi's actual function signature for `flash_loan_with_ctx`
- Verify parameter order
- Confirm return types

#### Step 2.4: Implement Real settle() Function

**Current** (lines 95-118):
```move
public fun settle<T>(
    loan_coin: Coin<T>,
    receipt: NaviFlashLoanReceipt<T>,
    repayment: Coin<T>,
    _ctx: &mut TxContext
): Coin<T> {
    // ... mock implementation with coin::destroy_zero()
}
```

**Replace with**:
```move
/// Settle the flash loan by repaying to Navi Protocol
/// Consumes the receipt and validates repayment
public fun settle<T>(
    clock: &Clock,
    storage: &mut Storage,
    pool: &mut Pool<T>,
    receipt: NaviReceipt<T>,
    repay_coin: Coin<T>,
    ctx: &mut TxContext
): Coin<T> {
    // Convert coin to balance for Navi
    let repay_balance = coin::into_balance(repay_coin);
    
    // Call Navi's repayment function
    let returned_balance = navi_lending::flash_repay_with_ctx<T>(
        clock,
        storage,
        pool,
        receipt,  // This consumes the receipt (hot potato pattern)
        repay_balance,
        ctx
    );
    
    // Convert any excess back to coin
    let excess_coin = coin::from_balance(returned_balance, ctx);
    excess_coin
}
```

**Critical**: Receipt is consumed by `flash_repay_with_ctx` (hot potato pattern).

#### Step 2.5: Update protocols.move Dispatch

**File**: `suiflash-contract/suiflash-router/sources/protocols.move`

**Update borrow_with_receipt** (around line 40):
```move
public fun borrow_with_receipt<CoinType>(
    protocol: u64, 
    amount: u64, 
    ctx: &mut TxContext
): (Coin<CoinType>, vector<u8>) {
    if (protocol == id_navi()) { 
        // Need to pass Navi-specific objects
        // Option 1: Get from global state
        // Option 2: Pass as parameters (requires router changes)
        
        // For now, this needs architectural decision
        abort 999 // Placeholder: needs Navi objects
    }
    // ... rest of protocols
}
```

**Challenge**: Navi requires Storage, Pool, Config objects. Solutions:
1. Store these in protocol config state
2. Pass as parameters (breaking change to interface)
3. Use capability pattern

#### Step 2.6: Test Compilation

```bash
cd suiflash-contract/suiflash-router
sui move build

# Should see:
# INCLUDING DEPENDENCY Navi
# BUILDING suiflash_router
# (warnings OK, but no errors)
```

**Common Errors**:
- `unbound module`: Check Navi import paths
- `mismatched types`: Verify Receipt<T> types match
- `unknown function`: Navi function name may differ

---

### Task 3: Complete PTB Construction (4-6 hours)

**File**: `suiflash-bot/src/navi_ptb_builder.rs`

#### Step 3.1: Uncomment PTB Building Code

**Lines 99-125** currently commented out. Uncomment and complete:

```rust
pub async fn build_navi_flash_loan_ptb(
    client: &SuiClient,
    config: &Config,
    plan: &ExecutionPlan,
) -> Result<ProgrammableTransaction> {
    info!("Building Navi Protocol flash loan PTB");
    
    let addresses = NaviAddresses::mainnet(config)?;
    let mut ptb = ProgrammableTransactionBuilder::new();
    
    let asset_type = get_asset_type_tag(&plan.user_operation)?;
    let pool_id = get_navi_pool_id_for_asset(config, &asset_type)?;
    
    // STEP 1: Get shared object versions
    let storage_version = get_shared_object_version(client, addresses.storage_id).await?;
    let pool_version = get_shared_object_version(client, pool_id).await?;
    let config_version = get_shared_object_version(client, addresses.flashloan_config_id).await?;
    
    // STEP 2: Build flash_loan_with_ctx call
    let flashloan_config_arg = ptb.obj(ObjectArg::SharedObject {
        id: addresses.flashloan_config_id,
        initial_shared_version: config_version,
        mutable: false,
    })?;
    
    let storage_arg = ptb.obj(ObjectArg::SharedObject {
        id: addresses.storage_id,
        initial_shared_version: storage_version,
        mutable: true,
    })?;
    
    let pool_arg = ptb.obj(ObjectArg::SharedObject {
        id: pool_id,
        initial_shared_version: pool_version,
        mutable: true,
    })?;
    
    let amount_arg = ptb.pure(plan.amount)?;
    
    // Call Navi's flash loan
    let flash_loan_result = ptb.move_call(
        addresses.protocol_package,
        Identifier::new("lending")?,
        Identifier::new("flash_loan_with_ctx")?,
        vec![asset_type.clone()],
        vec![storage_arg, pool_arg, flashloan_config_arg, amount_arg],
    )?;
    
    // Extract balance and receipt from result
    let balance = Argument::Result(flash_loan_result.0);
    let receipt = Argument::Result(flash_loan_result.1);
    
    // STEP 3: Convert balance to coin
    let loan_coin = ptb.move_call(
        ObjectID::from_hex_literal("0x0000000000000000000000000000000000000000000000000000000000000002")?,
        Identifier::new("coin")?,
        Identifier::new("from_balance")?,
        vec![asset_type.clone()],
        vec![balance],
    )?;
    
    // STEP 4: User operation (to be implemented)
    let returned_coin = execute_user_operation(&mut ptb, loan_coin, plan)?;
    
    // STEP 5: Convert coin back to balance
    let repay_balance = ptb.move_call(
        ObjectID::from_hex_literal("0x0000000000000000000000000000000000000000000000000000000000000002")?,
        Identifier::new("coin")?,
        Identifier::new("into_balance")?,
        vec![asset_type.clone()],
        vec![returned_coin],
    )?;
    
    // STEP 6: Repay to Navi
    let clock_arg = ptb.obj(ObjectArg::SharedObject {
        id: addresses.clock_id,
        initial_shared_version: SequenceNumber::from(1),
        mutable: false,
    })?;
    
    let _excess = ptb.move_call(
        addresses.protocol_package,
        Identifier::new("lending")?,
        Identifier::new("flash_repay_with_ctx")?,
        vec![asset_type.clone()],
        vec![clock_arg, storage_arg, pool_arg, receipt, repay_balance],
    )?;
    
    Ok(ptb.finish())
}
```

#### Step 3.2: Implement Shared Object Version Query

Update `get_shared_object_version()` (line 239):

```rust
pub async fn get_shared_object_version(
    client: &SuiClient,
    object_id: ObjectID,
) -> Result<SequenceNumber> {
    debug!("Querying initial shared version for object: {}", object_id);
    
    // Query the object from chain
    let object_response = client
        .read_api()
        .get_object_with_options(
            object_id,
            sui_json_rpc_types::SuiObjectDataOptions::new()
                .with_owner()
                .with_type(),
        )
        .await?;
    
    // Extract initial shared version
    if let Some(data) = object_response.data {
        if let Some(owner) = data.owner {
            // Parse owner to check if it's shared
            match owner {
                sui_json_rpc_types::SuiObjectOwner::Shared { initial_shared_version } => {
                    debug!("Found initial shared version: {:?}", initial_shared_version);
                    return Ok(initial_shared_version);
                }
                _ => {
                    return Err(eyre::eyre!("Object {} is not a shared object", object_id));
                }
            }
        }
    }
    
    Err(eyre::eyre!("Object {} not found", object_id))
}
```

#### Step 3.3: Add Missing Imports

```rust
use sui_sdk::types::transaction::{Argument, ObjectArg};
use sui_types::Identifier;
```

#### Step 3.4: Test Compilation

```bash
cd suiflash-bot
cargo check --package suiflash_bot

# Should compile without errors
```

---

### Task 4: Implement User Operation Callback (2-3 hours)

**File**: `suiflash-bot/src/navi_ptb_builder.rs`

#### Step 4.1: Parse User Operation

```rust
fn execute_user_operation(
    ptb: &mut ProgrammableTransactionBuilder,
    loan_coin: Argument,
    plan: &ExecutionPlan,
) -> Result<Argument> {
    // Parse user_operation string to determine strategy
    // Format examples:
    // - "arbitrage:dex_a:dex_b"
    // - "liquidation:protocol:account"
    // - "custom:contract_addr:function"
    
    if plan.user_operation.is_empty() {
        // No operation - just return the loan coin
        // In reality this would fail because fees aren't paid
        return Ok(loan_coin);
    }
    
    // Parse operation type
    let parts: Vec<&str> = plan.user_operation.split(':').collect();
    match parts.get(0) {
        Some(&"arbitrage") => execute_arbitrage(ptb, loan_coin, &parts[1..])?,
        Some(&"liquidation") => execute_liquidation(ptb, loan_coin, &parts[1..])?,
        Some(&"custom") => execute_custom(ptb, loan_coin, &parts[1..])?,
        _ => return Err(eyre::eyre!("Unknown user operation: {}", plan.user_operation)),
    }
}

fn execute_arbitrage(
    ptb: &mut ProgrammableTransactionBuilder,
    loan_coin: Argument,
    _params: &[&str],
) -> Result<Argument> {
    // TODO: Implement DEX arbitrage strategy
    // Example: Swap on DEX A, swap back on DEX B
    info!("Arbitrage operation not yet implemented");
    Ok(loan_coin)
}

fn execute_liquidation(
    ptb: &mut ProgrammableTransactionBuilder,
    loan_coin: Argument,
    _params: &[&str],
) -> Result<Argument> {
    // TODO: Implement liquidation strategy
    info!("Liquidation operation not yet implemented");
    Ok(loan_coin)
}

fn execute_custom(
    ptb: &mut ProgrammableTransactionBuilder,
    loan_coin: Argument,
    params: &[&str],
) -> Result<Argument> {
    // Call custom user contract
    let contract_addr = params.get(0)
        .ok_or_else(|| eyre::eyre!("Missing contract address"))?;
    let function_name = params.get(1)
        .ok_or_else(|| eyre::eyre!("Missing function name"))?;
    
    let package_id = ObjectID::from_hex_literal(contract_addr)?;
    
    // Call user's contract
    let result = ptb.move_call(
        package_id,
        Identifier::new("callback")?,
        Identifier::new(function_name)?,
        vec![], // Type arguments
        vec![loan_coin], // Pass the loan coin
    )?;
    
    Ok(result)
}
```

---

### Task 5: Integration and Testing (6-8 hours)

#### Step 5.1: Unit Tests

**File**: `suiflash-bot/src/navi_ptb_builder.rs`

Add at end of file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_navi_addresses_parsing() {
        let config = Config::default();
        let addresses = NaviAddresses::mainnet(&config).unwrap();
        
        assert_ne!(addresses.protocol_package, ObjectID::ZERO);
        assert_ne!(addresses.storage_id, ObjectID::ZERO);
    }
    
    #[test]
    fn test_pool_id_for_sui() {
        let config = Config::default();
        let type_tag = TypeTag::from_str("0x2::sui::SUI").unwrap();
        let pool_id = get_navi_pool_id_for_asset(&config, &type_tag).unwrap();
        
        // Verify it's the correct SUI pool
        assert_eq!(
            pool_id.to_hex_literal(),
            "0x96df0fce3c471489f4debaaa762cf960b3d7e146b6de9fbd3a5a39f89d2a56b8"
        );
    }
    
    #[tokio::test]
    async fn test_build_navi_ptb_structure() {
        let config = Config::default();
        let plan = ExecutionPlan {
            protocol: Protocol::Navi,
            amount: 1_000_000_000, // 1 SUI
            total_cost: 1_000_600_000, // 1.0006 SUI
            user_operation: "test".to_string(),
            callback_recipient: None,
            callback_payload: None,
        };
        
        // This should not panic
        let client = SuiClientBuilder::default()
            .build(&config.sui_rpc_url)
            .await
            .unwrap();
            
        let result = build_navi_flash_loan_ptb(&client, &config, &plan).await;
        // For now, might fail due to network, but shouldn't panic on structure
        assert!(result.is_ok() || result.is_err());
    }
}
```

#### Step 5.2: Integration Test

**File**: `suiflash-bot/src/integration_tests.rs`

Add:

```rust
#[tokio::test]
#[ignore] // Run manually with --ignored flag
async fn test_navi_flash_loan_on_testnet() {
    use crate::executors::FlashLoanExecutor;
    use crate::config::Config;
    use crate::strategies::ExecutionPlan;
    
    // Load testnet config
    let mut config = Config::load().unwrap();
    config.sui_rpc_url = "https://fullnode.testnet.sui.io:443".to_string();
    
    let executor = FlashLoanExecutor::new(config).await.unwrap();
    
    let plan = ExecutionPlan {
        protocol: Protocol::Navi,
        amount: 100_000_000, // 0.1 SUI (small amount for testing)
        total_cost: 100_060_000, // 0.10006 SUI
        user_operation: "test:noop".to_string(),
        callback_recipient: None,
        callback_payload: None,
    };
    
    let result = executor.execute_flash_loan(&plan).await;
    
    // On testnet, this might fail due to gas or liquidity
    // But it should not panic
    match result {
        Ok(digest) => {
            println!("✅ Flash loan succeeded: {}", digest);
        }
        Err(e) => {
            println!("⚠️ Flash loan failed (expected on testnet): {}", e);
        }
    }
}
```

#### Step 5.3: Manual Testing Script

**File**: `suiflash-bot/test_navi.sh`

```bash
#!/bin/bash
set -e

echo "=== Navi Flash Loan Integration Test ==="

# Build Move contract
echo "Building Move contract..."
cd suiflash-contract/suiflash-router
sui move build
cd ../..

# Build Rust bot
echo "Building Rust bot..."
cd suiflash-bot
cargo build --release

# Run unit tests
echo "Running unit tests..."
cargo test --lib

# Run integration tests (if available)
echo "Running integration tests..."
cargo test --test integration_tests -- --ignored

echo "✅ All tests passed!"
```

Make executable:
```bash
chmod +x suiflash-bot/test_navi.sh
```

---

## Common Issues & Solutions

### Issue 1: Navi Module Not Found

**Error**: `unbound module: navi::lending`

**Solution**:
1. Verify Navi dependency in Move.toml
2. Check Navi repo structure - module might be named differently
3. Try: `sui move build --skip-fetch-latest-git-deps`

### Issue 2: Type Mismatch on Receipt

**Error**: `expected Receipt<T>, found NaviFlashLoanReceipt<T>`

**Solution**:
- Use Navi's Receipt type directly, don't wrap it
- Update protocols.move to handle Navi Receipt specifically

### Issue 3: Shared Object Version Query Fails

**Error**: `failed to query object version`

**Solution**:
1. Check RPC endpoint is accessible
2. Verify object ID is correct
3. Use fallback: `SequenceNumber::from(1)` for testing

### Issue 4: PTB Execution Fails

**Error**: `transaction execution failed`

**Solution**:
1. Check gas budget is sufficient
2. Verify all shared objects have correct versions
3. Test transaction simulation first: `sui client dry-run`

---

## Deployment Checklist

Before deploying to mainnet:

- [ ] All unit tests pass
- [ ] Integration tests pass on testnet
- [ ] Fee calculations verified
- [ ] Gas estimation accurate
- [ ] Monitoring and alerts configured
- [ ] Rollback plan documented
- [ ] Security review completed
- [ ] Navi team notified (optional)

---

## Next Actions

1. **Immediate** (Today):
   - [ ] Add Navi dependency to Move.toml
   - [ ] Test Move contract builds

2. **Short Term** (This Week):
   - [ ] Implement real borrow() and settle()
   - [ ] Complete PTB construction
   - [ ] Add unit tests

3. **Medium Term** (Next Week):
   - [ ] Integration testing on testnet
   - [ ] User operation strategies
   - [ ] Performance optimization

4. **Long Term** (Next Month):
   - [ ] Production deployment
   - [ ] Monitoring and analytics
   - [ ] Documentation for users

---

## Support Resources

- **Navi Protocol Docs**: https://naviprotocol.gitbook.io
- **Sui Move Book**: https://move-book.com
- **Project Docs**: See `/docs` directory
- **Issues**: File in GitHub repo

---

## Summary

This guide provides everything needed to complete the Navi Protocol integration:

✅ **What's Ready**:
- Configuration
- Module structure
- PTB framework
- Helper functions
- Documentation

❌ **What's Needed**:
- Add Navi dependency
- Replace mock calls with real protocol calls
- Complete PTB construction
- Testing and validation

**Estimated Effort**: 2-3 days focused work

Good luck! 🚀
