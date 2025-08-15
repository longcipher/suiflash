# Navi Protocol Integration Guide

## Overview

This document describes the integration of Navi Protocol flash loans into the SuiFlash router contract. The integration provides a unified interface for flash loans across multiple protocols while maintaining protocol-specific optimizations.

## Architecture

### Components

1. **`navi_integration.move`** - Navi-specific adapter implementing flash loan logic
2. **`protocols.move`** - Protocol abstraction layer with unified dispatch
3. **`registry.move`** - On-chain registry for protocol adapter addresses
4. **`flash_router.move`** - Main entry points for flash loan operations

### Key Features

- **Receipt-based Settlement**: Proper Navi flash loan receipt handling
- **Fee Calculation**: Accurate 0.06% treasury fee computation
- **Error Handling**: Comprehensive validation and error reporting
- **Extensibility**: Easy addition of new protocols via adapter pattern

## Usage

### Basic Flash Loan

```move
use suiflash::flash_router;
use suiflash::protocols;

// Execute flash loan with SUI
entry fun flash_loan_coin<SUI>(
    cfg: &Config,
    protocol: u64,     // protocols::id_navi() = 0
    amount: u64,       // Amount to borrow (with decimals)
    recipient: address,
    payload: vector<u8>,
    ctx: &mut TxContext
);
```

### Protocol Selection

```move
// Available protocols
const PROTOCOL_NAVI: u64 = 0;    // 0.06% fee
const PROTOCOL_BUCKET: u64 = 1;  // 0.05% fee  
const PROTOCOL_SCALLOP: u64 = 2; // 0.09% fee

// Get protocol fee
let fee_bps = protocols::protocol_fee_bps(PROTOCOL_NAVI);
```

### Fee Calculation

```move
// Calculate Navi flash loan fee
let amount = 1_000_000_000; // 1 SUI (9 decimals)
let fee = navi_integration::calculate_fee(amount); // 600_000 units (0.06%)
let total_repayment = amount + fee;
```

## Implementation Details

### Navi Flash Loan Flow

1. **Borrow Phase**:
   - Call `navi_integration::borrow<T>(amount, ctx)`
   - Returns `(Coin<T>, NaviFlashLoanReceipt<T>)`
   - Receipt contains principal and fee information

2. **User Callback**:
   - Router calls user's callback function with borrowed funds
   - User performs arbitrary operations (arbitrage, liquidation, etc.)
   - Must return sufficient funds for repayment

3. **Settlement Phase**:
   - Call `navi_integration::settle<T>(loan, receipt, repay_coin, ctx)`
   - Validates repayment amount >= principal + fee
   - Destroys receipt and returns excess funds

### Error Handling

```move
// Common error cases
const E_INSUFFICIENT_REPAYMENT: u64 = 3;
const E_AMOUNT_TOO_LOW: u64 = 2;
const E_INVALID_PROTOCOL: u64 = 1;
```

## Configuration

### Protocol Addresses (To be updated)

```move
// Placeholder addresses - replace with actual Navi deployment
public fun flash_loan_config_id(): address { @0x0 }
public fun protocol_package(): address { @0x0 }

// Asset pool IDs
public fun sui_pool_id(): address { @0x0 }
public fun usdc_pool_id(): address { @0x0 }
public fun usdt_pool_id(): address { @0x0 }
```

### Asset Configuration

```move
// Navi asset IDs
public fun sui_asset_id(): u8 { 0 }
public fun usdc_asset_id(): u8 { 1 }
public fun usdt_asset_id(): u8 { 2 }
```

## Testing

Run the test suite to verify integration:

```bash
sui move test --package suiflash-router
```

Key test cases:

- Fee calculation accuracy
- Protocol ID dispatch
- Error handling
- Constant verification

## Deployment Steps

1. **Update Addresses**: Replace placeholder addresses with actual Navi deployment addresses
2. **Deploy Router**: Deploy the suiflash-router package  
3. **Initialize Registry**: Create and share ProtocolRegistry object
4. **Register Adapters**: Add protocol adapter addresses to registry
5. **Create Config**: Initialize router configuration with treasury and fees

## Integration Checklist

- [ ] Update Navi protocol addresses
- [ ] Set correct asset pool IDs
- [ ] Configure asset ID mappings
- [ ] Deploy and verify contracts
- [ ] Test flash loan flow end-to-end
- [ ] Update documentation with live addresses

## Future Enhancements

1. **Real Navi Integration**: Replace placeholder with actual Navi contract calls
2. **BCS Serialization**: Implement proper receipt serialization using BCS
3. **Additional Assets**: Add support for more token types
4. **Governance**: Add admin functions for protocol parameter updates
5. **Events**: Enhanced event emission for better observability

## References

- [Navi Protocol Documentation](https://naviprotocol.gitbook.io/navi-protocol-docs/)
- [Navi SDK](https://github.com/naviprotocol/navi-sdk)
- [Flash Loan Interface](https://github.com/naviprotocol/protocol-interface/blob/main/lending_core/sources/flash_loan.move)
- [SuiFlash Router Source](./sources/)
