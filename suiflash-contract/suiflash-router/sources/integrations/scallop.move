#[allow(duplicate_alias, lint(abort_without_constant))]
module suiflash::scallop_integration {
    //! Scallop Protocol Adapter
    //!
    //! Integrates with Scallop Protocol's flash loan functionality via:
    //! - borrow_flash_loan: Borrows liquidity and returns balance + FlashLoan hot potato
    //! - repay_flash_loan: Repays flash loan using FlashLoan hot potato and repayment coin
    //!
    //! Key Features:
    //! - Dynamic fee calculation based on protocol configuration
    //! - Base fee approximately 0.09% (9 basis points) - varies by asset and market conditions
    //! - Uses "hot potato" pattern for loan receipt
    //! - Must repay within same transaction block
    //! - Supports all active base assets in Scallop market
    //!
    //! Protocol Interface References:
    //! - Flash Loan: github.com/scallop-io/sui-lending-protocol/.../flash_loan.move
    //! - SDK: github.com/scallop-io/sui-scallop-sdk/blob/main/src/builders/coreBuilder.ts

    use std::type_name::{Self, TypeName};
    use sui::tx_context::TxContext;
    use sui::coin::{Self, Coin};
    use suiflash::errors;
    use sui::object::{Self, ID};

    /// Current Scallop protocol base fee: 0.09% (9 basis points)
    /// Note: Actual fees may vary by asset and market conditions
    /// Check Market configuration for current rates
    public fun fee_bps(): u64 { 9 }

    /// Scallop Protocol Flash Loan Hot Potato
    /// This represents the actual FlashLoan struct from Scallop protocol
    /// In production, this would be imported from protocol package
    public struct ScallopFlashLoanReceipt<phantom CoinType> has drop {
        /// Borrowed amount (principal)
        amount: u64,
        /// Protocol fee for this loan
        fee: u64,
        /// Asset type identifier
        asset_type: TypeName,
        /// Market object ID (for validation)
        market_id: ID,
    }

    /// Borrow via Scallop flash loan
    /// In production, this calls:
    /// `protocol::flash_loan::borrow_flash_loan<T>(version, market, amount, ctx)`
    public fun borrow<CoinType>(amount: u64, ctx: &mut TxContext): (Coin<CoinType>, ScallopFlashLoanReceipt<CoinType>) {
        // Placeholder: Create zero coin and receipt
        let coin = coin::zero<CoinType>(ctx);
        let fee = calculate_fee(amount);
        let asset_type = type_name::get<CoinType>();
        let market_id = object::id_from_address(@0x0); // Placeholder market ID
        
        let receipt = ScallopFlashLoanReceipt<CoinType> { 
            amount, 
            fee, 
            asset_type,
            market_id,
        };
        
        (coin, receipt)
    }

    /// Settle flash loan by repaying with hot potato
    /// In production, this calls:
    /// `protocol::flash_loan::repay_flash_loan<T>(version, market, repay_coin, loan_receipt)`
    public fun settle<CoinType>(
        loan_coin: Coin<CoinType>,
        receipt: ScallopFlashLoanReceipt<CoinType>,
        repay_coin: Coin<CoinType>,
        _ctx: &mut TxContext
    ): Coin<CoinType> {
        // Verify repayment amount includes principal + fee
        let required_amount = receipt.amount + receipt.fee;
        assert!(coin::value(&repay_coin) >= required_amount, errors::insufficient_repayment()); // Insufficient repayment
        
        // Verify asset type consistency
        let expected_type = type_name::get<CoinType>();
        assert!(receipt.asset_type == expected_type, errors::asset_type_mismatch()); // Asset type mismatch
        
        // In real implementation: call protocol::flash_loan::repay_flash_loan
        // For now: destroy receipt and return remaining balance
        let ScallopFlashLoanReceipt { amount: _, fee: _, asset_type: _, market_id: _ } = receipt;
        
        // Merge loan coin (unused in placeholder) and return repay coin
        coin::destroy_zero(loan_coin);
        repay_coin
    }

    /// Calculate Scallop protocol fee (0.09% base)
    /// Note: In production, this should query the actual market configuration
    /// as fees can vary by asset and market conditions
    public fun calculate_fee(amount: u64): u64 {
        amount * fee_bps() / 10_000
    }

    /// Get minimum repayment amount (principal + fee)
    public fun min_repayment<CoinType>(receipt: &ScallopFlashLoanReceipt<CoinType>): u64 {
        receipt.amount + receipt.fee
    }

    /// Get borrowed principal amount from receipt
    public fun receipt_amount<CoinType>(receipt: &ScallopFlashLoanReceipt<CoinType>): u64 {
        receipt.amount
    }

    /// Get protocol fee from receipt
    public fun receipt_fee<CoinType>(receipt: &ScallopFlashLoanReceipt<CoinType>): u64 {
        receipt.fee
    }

    /// Get asset type from receipt
    public fun receipt_asset_type<CoinType>(receipt: &ScallopFlashLoanReceipt<CoinType>): TypeName {
        receipt.asset_type
    }

    /// Create placeholder receipt for testing/serialization (internal use)
    public fun create_placeholder_receipt<CoinType>(
        amount: u64, 
        fee: u64,
        market_id: ID
    ): ScallopFlashLoanReceipt<CoinType> {
        ScallopFlashLoanReceipt<CoinType> { 
            amount, 
            fee, 
            asset_type: type_name::get<CoinType>(),
            market_id,
        }
    }

    /***********************************************************
    * SCALLOP PROTOCOL CONSTANTS (to be updated with real addresses)
    ***********************************************************/
    
    /// Placeholder addresses - replace with actual Scallop deployment addresses
    public fun version_object_id(): address { @0x0 }
    public fun market_object_id(): address { @0x0 }
    public fun protocol_package(): address { @0x0 }
    
    /// Market configuration for different assets (placeholders)
    /// In production, these would be queried from the actual Market object
    public fun sui_market_info(): (bool, u64, u64) { 
        (true, 0, 1_000_000_000_000) // (is_active, reserve_factor, max_borrow_cap)
    }
    
    public fun usdc_market_info(): (bool, u64, u64) { 
        (true, 0, 10_000_000_000) // (is_active, reserve_factor, max_borrow_cap) 
    }
    
    public fun usdt_market_info(): (bool, u64, u64) { 
        (true, 0, 10_000_000_000) // (is_active, reserve_factor, max_borrow_cap)
    }

    /// Asset type names for common Scallop assets
    public fun supported_assets(): vector<TypeName> {
        vector[
            type_name::get<sui::sui::SUI>(),
            // Add other supported asset types when available
        ]
    }

    /// Check if asset is supported for flash loans
    public fun is_asset_supported<CoinType>(): bool {
        let asset_type = type_name::get<CoinType>();
        
        // For now, support SUI as primary asset
        // In production, this would check the actual Scallop Market configuration
        asset_type == type_name::get<sui::sui::SUI>()
    }

    /// Get market configuration for specific asset type
    public fun get_market_config<CoinType>(): (bool, u64, u64) {
        let asset_type = type_name::get<CoinType>();
        
        // In production, this would query the actual Market object
        // For now, return default configuration based on common assets
        if (asset_type == type_name::get<sui::sui::SUI>()) {
            sui_market_info()
        } else {
            // Default for other assets (assuming USDC/USDT pattern)
            (true, 0, 10_000_000_000)
        }
    }

    /// Validate flash loan parameters before borrowing
    public fun validate_loan_request<CoinType>(amount: u64): bool {
        if (amount == 0) return false;
        if (!is_asset_supported<CoinType>()) return false;
        
        let (is_active, _reserve_factor, max_borrow) = get_market_config<CoinType>();
        if (!is_active) return false;
        if (amount > max_borrow) return false;
        
        true
    }

    /// Get estimated total cost for flash loan (principal + fee)
    public fun estimate_total_cost<CoinType>(amount: u64): u64 {
        if (!validate_loan_request<CoinType>(amount)) {
            return 0
        };
        
        amount + calculate_fee(amount)
    }
}
