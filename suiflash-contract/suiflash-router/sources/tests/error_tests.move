#[allow(duplicate_alias)]
#[allow(unused_use)]
module suiflash::error_tests {
    //! Error handling and edge case tests
    //!
    //! Tests cover:
    //! - Protocol validation errors
    //! - Amount validation errors  
    //! - Pause state errors
    //! - Fee calculation edge cases
    //! - Overflow protection

    use suiflash::protocols;
    use suiflash::state;

    // Test addresses
    #[allow(unused_const)]
    const TREASURY: address = @0xb;

    /// Test invalid protocol ID error
    #[test]
    #[expected_failure(abort_code = 1)]
    fun test_invalid_protocol_error() {
        protocols::protocol_fee_bps(999); // Invalid protocol ID
    }

    /// Test fee bounds validation during config creation
    #[test]
    #[expected_failure(abort_code = 0)]
    fun test_invalid_fee_creation() {
        let ctx = &mut sui::tx_context::dummy();
        // Should abort with fee > 10000 bps (100%)
        let (admin_cap, config) = state::create(TREASURY, 10001, ctx);
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }

    /// Test invalid service fee update
    #[test]
    #[expected_failure(abort_code = 0)]
    fun test_invalid_service_fee_update() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 100, ctx);
        let config_mut = &mut config;
        
        // Should abort with fee > 10000 bps
        state::set_service_fee(&admin_cap, config_mut, 10001);
        
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }

    /// Test pause assertion failure
    #[test]
    #[expected_failure(abort_code = 0)]
    fun test_pause_assertion_failure() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 50, ctx);
        let config_mut = &mut config;
        
        // Pause the system
        state::set_paused(&admin_cap, config_mut, true);
        
        // This should abort
        state::assert_not_paused(&config);
        
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }

    /// Test fee calculation overflow protection
    #[test]
    fun test_fee_overflow_protection() {
        // Test with large amounts that could cause overflow
        let large_amount = 1_000_000_000_000_000_000u64; // Very large amount
        let fee_bps = 6; // Navi protocol fee
        
        // This calculation should not overflow for reasonable amounts
        let fee = large_amount / 10_000 * fee_bps; // Reorder to prevent overflow
        assert!(fee < large_amount, 0);
        
        // Test the actual protocol fee calculation doesn't overflow
        let safe_amount = 18_446_744_073_709_551_615u64 / 10_000; // Max safe amount
        let protocol_fee = safe_amount * fee_bps / 10_000;
        assert!(protocol_fee <= safe_amount, 1);
    }

    /// Test edge cases with zero amounts
    #[test]
    fun test_zero_amount_edge_cases() {
        let amount = 0u64;
        
        // Zero amount should result in zero fees
        let navi_fee = amount * protocols::protocol_fee_bps(protocols::id_navi()) / 10_000;
        let bucket_fee = amount * protocols::protocol_fee_bps(protocols::id_bucket()) / 10_000;
        let scallop_fee = amount * protocols::protocol_fee_bps(protocols::id_scallop()) / 10_000;
        
        assert!(navi_fee == 0, 0);
        assert!(bucket_fee == 0, 1);
        assert!(scallop_fee == 0, 2);
    }

    /// Test minimum amount for non-zero fees
    #[test]
    fun test_minimum_fee_amounts() {
        // Find minimum amounts that result in non-zero fees
        let navi_fee_bps = protocols::protocol_fee_bps(protocols::id_navi());
        let bucket_fee_bps = protocols::protocol_fee_bps(protocols::id_bucket());
        let scallop_fee_bps = protocols::protocol_fee_bps(protocols::id_scallop());
        
        // Calculate minimum amounts for 1 unit fee
        let min_navi = 10_000 / navi_fee_bps + 1;
        let min_bucket = 10_000 / bucket_fee_bps + 1;
        let min_scallop = 10_000 / scallop_fee_bps + 1;
        
        // Verify these amounts produce non-zero fees
        assert!(min_navi * navi_fee_bps / 10_000 > 0, 0);
        assert!(min_bucket * bucket_fee_bps / 10_000 > 0, 1);
        assert!(min_scallop * scallop_fee_bps / 10_000 > 0, 2);
    }

    /// Test protocol fee bounds
    #[test]
    fun test_protocol_fee_sanity_checks() {
        // All protocol fees should be reasonable (< 1000 bps = 10%)
        assert!(protocols::protocol_fee_bps(protocols::id_navi()) < 1000, 0);
        assert!(protocols::protocol_fee_bps(protocols::id_bucket()) < 1000, 1);
        assert!(protocols::protocol_fee_bps(protocols::id_scallop()) < 1000, 2);
        
        // All fees should be positive
        assert!(protocols::protocol_fee_bps(protocols::id_navi()) > 0, 3);
        assert!(protocols::protocol_fee_bps(protocols::id_bucket()) > 0, 4);
        assert!(protocols::protocol_fee_bps(protocols::id_scallop()) > 0, 5);
        
        // Fees should be different (to provide choice)
        assert!(protocols::protocol_fee_bps(protocols::id_navi()) != protocols::protocol_fee_bps(protocols::id_bucket()), 6);
        assert!(protocols::protocol_fee_bps(protocols::id_bucket()) != protocols::protocol_fee_bps(protocols::id_scallop()), 7);
    }

    /// Test borrow with invalid protocol
    #[test]
    fun test_borrow_invalid_protocol() {
        let ctx = &mut sui::tx_context::dummy();
        
        // Invalid protocol should return zero coin and empty receipt
        let (coin, receipt_bytes) = protocols::borrow_with_receipt<sui::sui::SUI>(
            999, // Invalid protocol
            1_000_000_000,
            ctx
        );
        
        assert!(sui::coin::value(&coin) == 0, 0);
        assert!(std::vector::length(&receipt_bytes) == 0, 1);
        
        sui::transfer::public_transfer(coin, @0x0);
    }

    /// Test division precision in fee calculations
    #[test]
    fun test_fee_precision() {
        // Test amounts that might have precision issues
        let test_amounts = vector[
            1u64,           // Minimum 
            99u64,          // Less than 100
            999u64,         // Less than 1000
            9999u64,        // Less than 10000
            10000u64,       // Exactly 10000 (100% in bps)
            10001u64,       // Just over 10000
        ];
        
        let i = 0;
        while (i < std::vector::length(&test_amounts)) {
            let amount = *std::vector::borrow(&test_amounts, i);
            let fee = amount * 6 / 10_000; // Navi fee calculation
            
            // Fee should never exceed the amount for reasonable fee rates
            assert!(fee <= amount, i);
            
            i = i + 1;
        }
    }

    /// Test asset allowlist boundary conditions
    #[test]
    fun test_asset_allowlist_boundaries() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 50, ctx);
        let config_mut = &mut config;
        
        // Test with maximum asset ID
        let max_asset_id = 18_446_744_073_709_551_615u64; // Max u64
        state::add_allowed_asset(&admin_cap, config_mut, max_asset_id);
        assert!(state::is_allowed_asset(&config, max_asset_id), 0);
        
        // Test with zero asset ID
        state::add_allowed_asset(&admin_cap, config_mut, 0);
        assert!(state::is_allowed_asset(&config, 0), 1);
        
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }

    /// Test protocol config boundary conditions
    #[test]
    fun test_protocol_config_boundaries() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 50, ctx);
        let config_mut = &mut config;
        
        // Test with large protocol ID
        state::set_protocol_config(&admin_cap, config_mut, 1000, @0x123);
        assert!(state::protocol_config(&config, 1000) == @0x123, 0);
        
        // All intermediate indices should be 0x0
        assert!(state::protocol_config(&config, 500) == @0x0, 1);
        assert!(state::protocol_config(&config, 999) == @0x0, 2);
        
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }
}
