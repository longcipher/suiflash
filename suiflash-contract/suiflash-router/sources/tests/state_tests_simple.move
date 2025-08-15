#[allow(duplicate_alias)]
module suiflash::state_tests_simple {
    //! Basic unit tests for state management module
    //!
    //! Tests cover:
    //! - Config creation and validation
    //! - Fee settings and bounds checking
    //! - Asset allowlist management
    //! - Protocol configuration storage

    use suiflash::state;

    // Test addresses
    const TREASURY: address = @0xb;

    /// Test config creation with valid parameters
    #[test]
    fun test_config_creation() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 100, ctx);
        
        // Verify initial state
        assert!(state::service_fee_bps(&config) == 100, 0);
        assert!(!state::is_allowed_asset(&config, 1), 1);
        
        // Test pause functionality
        state::assert_not_paused(&config);
        
        // Clean up - transfer to dummy address
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }

    /// Test fee bounds validation during creation
    #[test]
    fun test_valid_fee_boundaries() {
        let ctx = &mut sui::tx_context::dummy();
        
        // Test minimum fee (0)
        let (admin_cap1, config1) = state::create(TREASURY, 0, ctx);
        assert!(state::service_fee_bps(&config1) == 0, 0);
        
        // Test maximum fee (10000 bps = 100%)
        let (admin_cap2, config2) = state::create(TREASURY, 10000, ctx);
        assert!(state::service_fee_bps(&config2) == 10000, 1);
        
        sui::transfer::public_transfer(admin_cap1, @0x0);
        sui::transfer::public_transfer(config1, @0x0);
        sui::transfer::public_transfer(admin_cap2, @0x0);
        sui::transfer::public_transfer(config2, @0x0);
    }

    /// Test service fee updates
    #[test]
    fun test_service_fee_updates() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 100, ctx);
        let config_ref = &mut config;
        
        // Update fee to valid value
        state::set_service_fee(&admin_cap, config_ref, 200);
        assert!(state::service_fee_bps(config_ref) == 200, 0);
        
        // Update to maximum allowed fee
        state::set_service_fee(&admin_cap, config_ref, 10000);
        assert!(state::service_fee_bps(config_ref) == 10000, 1);
        
        // Update to zero
        state::set_service_fee(&admin_cap, config_ref, 0);
        assert!(state::service_fee_bps(config_ref) == 0, 2);
        
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }

    /// Test asset allowlist management
    #[test]
    fun test_asset_allowlist() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 50, ctx);
        let config_ref = &mut config;
        
        // Initially no assets allowed
        assert!(!state::is_allowed_asset(config_ref, 1), 0);
        assert!(!state::is_allowed_asset(config_ref, 2), 1);
        
        // Add assets to allowlist
        state::add_allowed_asset(&admin_cap, config_ref, 1);
        state::add_allowed_asset(&admin_cap, config_ref, 2);
        
        // Verify assets are now allowed
        assert!(state::is_allowed_asset(config_ref, 1), 2);
        assert!(state::is_allowed_asset(config_ref, 2), 3);
        
        // Non-added asset should still not be allowed
        assert!(!state::is_allowed_asset(config_ref, 3), 4);
        
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }

    /// Test protocol configuration storage
    #[test]
    fun test_protocol_config_storage() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 50, ctx);
        let config_ref = &mut config;
        
        // Initially protocol configs should return 0x0
        assert!(state::protocol_config(config_ref, 0) == @0x0, 0);
        assert!(state::protocol_config(config_ref, 1) == @0x0, 1);
        
        // Set protocol configs
        state::set_protocol_config(&admin_cap, config_ref, 0, @0x123);
        state::set_protocol_config(&admin_cap, config_ref, 2, @0x456); // Skip index 1
        
        // Verify configs
        assert!(state::protocol_config(config_ref, 0) == @0x123, 2);
        assert!(state::protocol_config(config_ref, 1) == @0x0, 3); // Should be placeholder
        assert!(state::protocol_config(config_ref, 2) == @0x456, 4);
        
        // Out of bounds should return 0x0
        assert!(state::protocol_config(config_ref, 10) == @0x0, 5);
        
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }

    /// Test protocol config vector growth
    #[test]
    fun test_protocol_config_vector_growth() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 50, ctx);
        let config_ref = &mut config;
        
        // Set config at a high index to test vector growth
        state::set_protocol_config(&admin_cap, config_ref, 5, @0x789);
        
        // All intermediate indices should be filled with 0x0
        assert!(state::protocol_config(config_ref, 0) == @0x0, 0);
        assert!(state::protocol_config(config_ref, 1) == @0x0, 1);
        assert!(state::protocol_config(config_ref, 2) == @0x0, 2);
        assert!(state::protocol_config(config_ref, 3) == @0x0, 3);
        assert!(state::protocol_config(config_ref, 4) == @0x0, 4);
        assert!(state::protocol_config(config_ref, 5) == @0x789, 5);
        
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }

    /// Test asset allowlist edge cases
    #[test]
    fun test_asset_allowlist_edge_cases() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 50, ctx);
        let config_ref = &mut config;
        
        // Test with asset ID 0
        state::add_allowed_asset(&admin_cap, config_ref, 0);
        assert!(state::is_allowed_asset(config_ref, 0), 0);
        
        // Test with large asset ID
        state::add_allowed_asset(&admin_cap, config_ref, 999999);
        assert!(state::is_allowed_asset(config_ref, 999999), 1);
        
        // Test duplicate additions (should not cause issues)
        state::add_allowed_asset(&admin_cap, config_ref, 0);
        assert!(state::is_allowed_asset(config_ref, 0), 2);
        
        sui::transfer::public_transfer(admin_cap, @0x0);
        sui::transfer::public_transfer(config, @0x0);
    }
}
