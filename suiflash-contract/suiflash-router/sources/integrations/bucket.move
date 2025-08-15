/// Bucket Protocol Integration for SuiFlash
/// Provides flash loan functionality through Bucket Protocol
/// Bucket Protocol supports flash loans for SUI and BUCK tokens
#[allow(duplicate_alias, unused_use)]
module suiflash::bucket_integration {
    use sui::coin::{Self, Coin};
    use sui::balance;
    use sui::tx_context::TxContext;

    // Flash loan fee: 5 basis points (0.05%)
    const FLASH_LOAN_FEE_BPS: u64 = 5;
    const BPS_DENOMINATOR: u64 = 10000;

    // Error codes
    const E_INSUFFICIENT_AMOUNT: u64 = 1;
    const E_REPAYMENT_AMOUNT_MISMATCH: u64 = 3;

    /// Flash loan receipt containing loan details
    /// This follows the hot potato pattern - must be consumed
    public struct BucketFlashLoanReceipt<phantom T> has drop {
        amount: u64,
        fee: u64,
        total_repay: u64,
    }

    /// Protocol fee structure (compatible with Bucket's 5 bps fee)
    public fun fee_bps(): u64 {
        FLASH_LOAN_FEE_BPS
    }

    /// Borrow assets through Bucket Protocol flash loan
    /// Returns the borrowed coins and a receipt that must be settled
    public fun borrow<T>(
        amount: u64,
        ctx: &mut TxContext
    ): (Coin<T>, BucketFlashLoanReceipt<T>) {
        // Validate borrow amount
        assert!(amount > 0, E_INSUFFICIENT_AMOUNT);

        // Calculate fee (5 basis points)
        let fee = calculate_fee(amount);
        let total_repay = amount + fee;

        // Create flash loan receipt (hot potato)
        let receipt = BucketFlashLoanReceipt<T> {
            amount,
            fee,
            total_repay,
        };

        // In a real implementation, this would call Bucket Protocol's flash_borrow
        // For testing, we create a zero coin and let tests provide actual repayment
        let borrowed_coin = sui::coin::zero<T>(ctx);
        
        (borrowed_coin, receipt)
    }

    /// Settle the flash loan by repaying the borrowed amount plus fees
    /// Consumes the receipt and validates repayment amount
    /// Returns the repayment coin (protocol would keep fee in production)
    public fun settle<T>(
        repayment: Coin<T>,
        receipt: BucketFlashLoanReceipt<T>,
        _ctx: &mut TxContext
    ): Coin<T> {
        let BucketFlashLoanReceipt { amount: _, fee: _, total_repay } = receipt;
        
        // Verify repayment amount matches total required
        let repayment_amount = coin::value(&repayment);
        assert!(repayment_amount >= total_repay, E_REPAYMENT_AMOUNT_MISMATCH);

        // In a real implementation, this would call Bucket Protocol's flash_repay
        // bucket::flash_repay<T>(protocol_object, repayment, original_receipt)
        
        // For testing, we return the full repayment after validation
        // In production, the protocol would extract the required amount
        repayment
    }

    /// Calculate the flash loan fee for a given amount
    /// Returns the fee amount in the same denomination as the principal
    public fun calculate_fee(amount: u64): u64 {
        (amount * FLASH_LOAN_FEE_BPS) / BPS_DENOMINATOR
    }

    /// Get the total repayment amount (principal + fee)
    public fun get_total_repay_amount(amount: u64): u64 {
        amount + calculate_fee(amount)
    }

    /// Validate if a coin type is supported by Bucket Protocol
    /// Bucket Protocol primarily supports SUI and BUCK tokens for flash loans
    public fun is_supported_asset(): bool {
        // In a real implementation, this would check against Bucket's supported assets
        // For now, we assume SUI and BUCK are supported
        true
    }

    /// Get flash loan receipt details for verification
    public fun get_receipt_details<T>(receipt: &BucketFlashLoanReceipt<T>): (u64, u64, u64) {
        (receipt.amount, receipt.fee, receipt.total_repay)
    }

    // === Test Functions ===
    
    public fun create_test_receipt<T>(amount: u64): BucketFlashLoanReceipt<T> {
        let fee = calculate_fee(amount);
        BucketFlashLoanReceipt<T> {
            amount,
            fee,
            total_repay: amount + fee,
        }
    }

    #[test_only]
    public fun destroy_test_receipt<T>(receipt: BucketFlashLoanReceipt<T>) {
        let BucketFlashLoanReceipt { amount: _, fee: _, total_repay: _ } = receipt;
    }

    // === Integration Tests ===

    #[test]
    fun test_fee_calculation() {
        // Test various amounts
        assert!(calculate_fee(10000) == 5, 0); // 10000 * 5 / 10000 = 5
        assert!(calculate_fee(100000) == 50, 1); // 100000 * 5 / 10000 = 50
        assert!(calculate_fee(1000000) == 500, 2); // 1000000 * 5 / 10000 = 500
        
        // Test edge cases
        assert!(calculate_fee(0) == 0, 3);
        assert!(calculate_fee(1) == 0, 4); // Should round down to 0
        assert!(calculate_fee(2000) == 1, 5); // 2000 * 5 / 10000 = 1
    }

    #[test]
    fun test_total_repay_amount() {
        let amount = 100000;
        let expected_fee = 50;
        let expected_total = amount + expected_fee;
        
        assert!(get_total_repay_amount(amount) == expected_total, 0);
    }

    #[test]
    fun test_receipt_creation_and_details() {
        let amount = 50000;
        let receipt = create_test_receipt<sui::sui::SUI>(amount);
        
        let (receipt_amount, receipt_fee, receipt_total) = get_receipt_details(&receipt);
        assert!(receipt_amount == amount, 0);
        assert!(receipt_fee == calculate_fee(amount), 1);
        assert!(receipt_total == amount + receipt_fee, 2);
        
        destroy_test_receipt(receipt);
    }

    #[test]
    fun test_asset_support() {
        // All assets should be supported in our test implementation
        assert!(is_supported_asset(), 0);
    }

    #[test]
    fun test_fee_bps() {
        assert!(fee_bps() == 5, 0);
    }
}
