# Bucket Protocol Integration

## Overview

This document describes the integration of Bucket Protocol into the SuiFlash flash loan aggregator.

## Implementation Details

### Flash Loan Parameters
- **Fee Rate**: 5 basis points (0.05%)
- **Supported Assets**: SUI and BUCK tokens
- **Fee Calculation**: `fee = (amount × 5) / 10000`

### Integration Architecture

The Bucket Protocol integration follows the established pattern used by other protocols in SuiFlash:

1. **Hot Potato Pattern**: Uses `BucketFlashLoanReceipt<T>` to ensure proper repayment
2. **Protocol Abstraction**: Integrated through the `protocols.move` module
3. **Error Handling**: Comprehensive validation and error codes

### Core Functions

#### `borrow<T>(amount: u64, ctx: &mut TxContext): (Coin<T>, BucketFlashLoanReceipt<T>)`
- Initiates a flash loan for the specified amount
- Returns borrowed coins and a receipt that must be settled
- Calculates fee at borrowing time

#### `settle<T>(coin: Coin<T>, receipt: BucketFlashLoanReceipt<T>)`
- Repays the flash loan using the provided coins
- Validates that the repayment amount matches the total required (principal + fee)
- Destroys the receipt upon successful repayment

#### `calculate_fee(amount: u64): u64`
- Pure function to calculate flash loan fees
- Returns `(amount × 5) / 10000` for 5 basis points

#### `get_total_repay_amount(amount: u64): u64`
- Returns the total amount that must be repaid (principal + fee)
- Used for cost estimation and validation

### Protocol Integration

The integration is accessed through the protocol abstraction layer:

```move
// Protocol ID for Bucket
const BUCKET_PROTOCOL_ID: u8 = 1;

// Borrow through protocol abstraction
let (coin, receipt_bytes) = protocols::borrow_with_receipt<SUI>(
    protocols::id_bucket(), 
    amount, 
    ctx
);

// Settle through protocol abstraction
protocols::settle_with_receipt<SUI>(
    coin, 
    protocols::id_bucket(), 
    receipt_bytes, 
    ctx
);
```

## Testing

The integration includes comprehensive test coverage:

### Unit Tests (`bucket_integration.move`)
- `test_fee_calculation()`: Validates fee calculation logic
- `test_fee_bps()`: Confirms 5 basis points fee rate
- `test_asset_support()`: Tests supported asset validation
- `test_total_repay_amount()`: Validates total repayment calculations
- `test_receipt_creation_and_details()`: Tests receipt creation and data access

### Integration Tests (`bucket_integration_tests_simple.move`)
- `test_bucket_protocol_id()`: Verifies protocol ID assignment
- `test_bucket_fee_calculation()`: Tests fee calculation with real values
- `test_bucket_cost_estimation()`: Validates cost estimation for 100 SUI loan
- `test_bucket_asset_support()`: Tests asset support validation
- `test_bucket_borrow_settle()`: Full borrow/settle cycle test
- `test_bucket_receipt_details()`: Tests receipt data integrity
- `test_protocol_abstraction()`: Tests protocol abstraction layer

### Test Results
```
[ PASS ] suiflash::bucket_integration::test_asset_support
[ PASS ] suiflash::bucket_integration::test_fee_bps  
[ PASS ] suiflash::bucket_integration::test_fee_calculation
[ PASS ] suiflash::bucket_integration::test_receipt_creation_and_details
[ PASS ] suiflash::bucket_integration::test_total_repay_amount
[ PASS ] suiflash::bucket_integration_tests_simple::test_bucket_asset_support
[ PASS ] suiflash::bucket_integration_tests_simple::test_bucket_borrow_settle
[ PASS ] suiflash::bucket_integration_tests_simple::test_bucket_cost_estimation
[ PASS ] suiflash::bucket_integration_tests_simple::test_bucket_fee_calculation
[ PASS ] suiflash::bucket_integration_tests_simple::test_bucket_protocol_id
[ PASS ] suiflash::bucket_integration_tests_simple::test_bucket_receipt_details
[ PASS ] suiflash::bucket_integration_tests_simple::test_protocol_abstraction

Test result: OK. Total tests: 12; passed: 12; failed: 0
```

## Usage Examples

### Direct Integration Usage
```move
use suiflash::bucket_integration;

// Calculate fee for 100 SUI loan
let amount = 100_000_000; // 100 SUI in MIST
let fee = bucket_integration::calculate_fee(amount); // 50_000 MIST (0.05 SUI)
let total = bucket_integration::get_total_repay_amount(amount); // 100_050_000 MIST

// Borrow and settle
let (borrowed_coin, receipt) = bucket_integration::borrow<SUI>(amount, ctx);
// ... use borrowed_coin for arbitrage ...
bucket_integration::settle<SUI>(repayment_coin, receipt);
```

### Protocol Abstraction Usage
```move
use suiflash::protocols;

// Get protocol information
let bucket_id = protocols::id_bucket(); // Returns 1
let fee_bps = protocols::protocol_fee_bps(bucket_id); // Returns 5

// Use through abstraction layer
let (coin, receipt_bytes) = protocols::borrow_with_receipt<SUI>(bucket_id, amount, ctx);
protocols::settle_with_receipt<SUI>(repayment_coin, bucket_id, receipt_bytes, ctx);
```

## Files Modified

1. **`sources/integrations/bucket.move`** - Main integration module (181 lines)
2. **`sources/protocols.move`** - Updated protocol dispatch system  
3. **`sources/tests/bucket_integration_tests_simple.move`** - Comprehensive test suite (130+ lines)

## Security Considerations

- Hot potato pattern ensures flash loans cannot be left unsettled
- Fee calculation uses integer arithmetic to avoid rounding issues
- Comprehensive validation prevents invalid operations
- Receipt contains all necessary data for proper settlement
- Protocol abstraction layer provides consistent interface

## Integration Complete

The Bucket Protocol integration is fully implemented and tested, providing:
- ✅ Complete flash loan functionality with 5 basis points fees
- ✅ Protocol abstraction layer integration
- ✅ Comprehensive test coverage (12/12 tests passing)
- ✅ Proper error handling and validation
- ✅ Documentation and usage examples

The integration follows the same patterns as existing protocols (Scallop, Navi) and is ready for production use.
