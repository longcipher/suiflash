#[allow(duplicate_alias)]
module suiflash::protocols {
    /*! Protocol Abstraction Layer

    This module centralises protocol IDs and dispatch helpers so the flash router
    does not need to hard‑code per‑protocol logic beyond a single match. Adding a
    new protocol requires ONLY:
      1. Creating an `<name>_integration` module implementing the minimal adapter
         functions declared below.
      2. Adding its constant ID & branch inside `dispatch_*` helpers.

    The adapter pattern keeps a stable surface for the router while enabling
    richer per‑protocol logic (e.g. different fee computation rules, delegated
    callback style flash loans, etc.). For now all adapters expose a uniform
    fee basis points function; future extensions can include borrow/repay hooks
    when integrating real protocol APIs.

    NOTE: Move lacks first‑class traits; we emulate an interface contract by
    documenting the required public functions each adapter module must expose:
        public fun fee_bps(): u64;
        // (optional, future) public fun supports_delegated_callback(): bool;
        // (optional, future) public fun borrow<CoinType>(amount: u64, ctx: &mut TxContext): Coin<CoinType>;
        // (optional, future) public fun settle<CoinType>(returned: Coin<CoinType>, required_total: u64, ctx: &mut TxContext);
    */

    use std::vector;
    use sui::tx_context::TxContext;
    use sui::coin::{Self, Coin};
    use suiflash::errors;
    use suiflash::navi_integration;
    use suiflash::bucket_integration;
    use suiflash::scallop_integration;

    /// Protocol numeric identifiers (helpers return stable IDs instead of using const declarations to avoid dialect limitations).
    public fun id_navi(): u64 { 0 }
    public fun id_bucket(): u64 { 1 }
    public fun id_scallop(): u64 { 2 }

    /// Return protocol fee in basis points (parts per 10_000) or abort if unknown.
    public fun protocol_fee_bps(protocol: u64): u64 {
        if (protocol == id_navi()) { navi_integration::fee_bps() }
        else if (protocol == id_bucket()) { bucket_integration::fee_bps() }
        else if (protocol == id_scallop()) { scallop_integration::fee_bps() }
        else { abort errors::invalid_protocol() }
    }

    /// Dispatch borrow to adapter with receipt handling for all protocols
    public fun borrow_with_receipt<CoinType>(protocol: u64, amount: u64, ctx: &mut TxContext): (Coin<CoinType>, vector<u8>) {
        if (protocol == id_navi()) { 
            let (coin, receipt) = navi_integration::borrow<CoinType>(amount, ctx);
            // Serialize receipt for generic handling (placeholder - use BCS in production)
            let receipt_bytes = navi_receipt_to_bytes(receipt);
            (coin, receipt_bytes)
        }
        else if (protocol == id_bucket()) {
            let (coin, receipt) = bucket_integration::borrow<CoinType>(amount, ctx);
            // Serialize Bucket receipt for generic handling
            let receipt_bytes = bucket_receipt_to_bytes(receipt);
            (coin, receipt_bytes)
        }
        else if (protocol == id_scallop()) {
            let (coin, receipt) = scallop_integration::borrow<CoinType>(amount, ctx);
            // Serialize Scallop receipt for generic handling
            let receipt_bytes = scallop_receipt_to_bytes(receipt);
            (coin, receipt_bytes)
        }
        else { 
            (coin::zero<CoinType>(ctx), vector::empty<u8>())
        }
    }

    /// Dispatch settle to adapter with receipt handling  
    public fun settle_with_receipt<CoinType>(
        protocol: u64, 
        loan: Coin<CoinType>, 
        receipt_bytes: vector<u8>,
        repay_coin: Coin<CoinType>, 
        ctx: &mut TxContext
    ): Coin<CoinType> {
        if (protocol == id_navi()) { 
            let receipt = navi_receipt_from_bytes<CoinType>(receipt_bytes);
            navi_integration::settle<CoinType>(loan, receipt, repay_coin, ctx)
        }
        else if (protocol == id_bucket()) {
            let receipt = bucket_receipt_from_bytes<CoinType>(receipt_bytes);
            let returned = bucket_integration::settle<CoinType>(repay_coin, receipt, ctx);
            coin::destroy_zero(loan);
            returned
        }
        else if (protocol == id_scallop()) {
            let receipt = scallop_receipt_from_bytes<CoinType>(receipt_bytes);
            scallop_integration::settle<CoinType>(loan, receipt, repay_coin, ctx)
        }
        else { 
            // For other protocols, ignore receipt and return repay_coin
            coin::destroy_zero(loan);
            repay_coin
        }
    }

    /// Legacy borrow_coin for backward compatibility (simplified interface without receipts)
    public fun borrow_coin<CoinType>(protocol: u64, amount: u64, ctx: &mut TxContext): Coin<CoinType> {
        let (coin, _receipt_bytes) = borrow_with_receipt<CoinType>(protocol, amount, ctx);
        coin
    }

    /// Legacy settle_coin for backward compatibility
    public fun settle_coin<CoinType>(protocol: u64, loan: Coin<CoinType>, _required_total: u64, _ctx: &mut TxContext): Coin<CoinType> {
        if (protocol == id_navi()) { 
            // For legacy interface, we can't properly settle without receipt, so just return loan
            loan
        }
        else { loan }
    }

    /// Serialize Navi receipt to bytes (placeholder - use BCS serialization in production)
    fun navi_receipt_to_bytes<CoinType>(_receipt: navi_integration::NaviFlashLoanReceipt<CoinType>): vector<u8> {
        // Placeholder serialization - in production use BCS
        vector::empty<u8>() // Return empty for now
    }

    /// Deserialize bytes to Navi receipt (placeholder)
    fun navi_receipt_from_bytes<CoinType>(_bytes: vector<u8>): navi_integration::NaviFlashLoanReceipt<CoinType> {
        // Placeholder deserialization - in production use BCS
        navi_integration::create_placeholder_receipt<CoinType>(0, 0)
    }

    /// Serialize Bucket receipt to bytes (placeholder - use BCS serialization in production)
    fun bucket_receipt_to_bytes<CoinType>(_receipt: bucket_integration::BucketFlashLoanReceipt<CoinType>): vector<u8> {
        // Placeholder serialization - in production use BCS
        vector::empty<u8>() // Return empty for now
    }

    /// Deserialize bytes to Bucket receipt (placeholder)
    fun bucket_receipt_from_bytes<CoinType>(_bytes: vector<u8>): bucket_integration::BucketFlashLoanReceipt<CoinType> {
        // Placeholder deserialization - in production use BCS
        bucket_integration::create_test_receipt<CoinType>(0)
    }

    /// Serialize Scallop receipt to bytes (placeholder - use BCS serialization in production)
    fun scallop_receipt_to_bytes<CoinType>(_receipt: scallop_integration::ScallopFlashLoanReceipt<CoinType>): vector<u8> {
        // Placeholder serialization - in production use BCS
        vector::empty<u8>() // Return empty for now
    }

    /// Deserialize bytes to Scallop receipt (placeholder)
    fun scallop_receipt_from_bytes<CoinType>(_bytes: vector<u8>): scallop_integration::ScallopFlashLoanReceipt<CoinType> {
        // Placeholder deserialization - in production use BCS
        use sui::object;
        scallop_integration::create_placeholder_receipt<CoinType>(0, 0, object::id_from_address(@0x0))
    }
}
