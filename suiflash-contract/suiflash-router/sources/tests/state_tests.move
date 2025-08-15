#[test_only]
module suiflash::state_tests {
    use sui::test_utils;
    use suiflash::state::{Self, Config, AdminCap};

    const ADMIN: address = @0xa;
    const TREASURY: address = @0xb;

    #[test]
    fun test_create_config() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 100, ctx);
        
        // Test basic properties
        assert!(state::service_fee_bps(&config) == 100, 0);
        
        // Clean up
        test_utils::destroy(admin_cap);
        test_utils::destroy(config);
    }

    #[test]
    fun test_protocol_config() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 100, ctx);
        
        // Test default protocol config (should be 0x0)
        assert!(state::protocol_config(&config, 0) == @0x0, 0);
        
        // Clean up
        test_utils::destroy(admin_cap);
        test_utils::destroy(config);
    }

    #[test]
    fun test_pause_functionality() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 100, ctx);
        
        // Should not be paused initially
        state::assert_not_paused(&config);
        
        // Clean up
        test_utils::destroy(admin_cap);
        test_utils::destroy(config);
    }

    #[test]
    #[expected_failure(abort_code = 0)]
    fun test_invalid_fee() {
        let ctx = &mut sui::tx_context::dummy();
        let (admin_cap, config) = state::create(TREASURY, 10_001, ctx); // Should fail
        
        test_utils::destroy(admin_cap);
        test_utils::destroy(config);
    }
}
