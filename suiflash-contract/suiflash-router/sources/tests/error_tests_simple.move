#[allow(unused_use)]
#[test_only]
module suiflash::error_tests_simple {
    use sui::test_utils;
    use suiflash::state;

    #[test]
    #[expected_failure]
    fun test_invalid_fee_rate() {
        let ctx = &mut sui::tx_context::dummy();
        // This should fail because fee rate > 10000 (100%)
        let (admin_cap, config) = state::create(@0xb, 10_001, ctx);
        
        test_utils::destroy(admin_cap);
        test_utils::destroy(config);
    }

    #[test]
    fun test_valid_fee_rate() {
        let ctx = &mut sui::tx_context::dummy();
        // This should succeed
        let (admin_cap, config) = state::create(@0xb, 100, ctx);
        
        assert!(state::service_fee_bps(&config) == 100, 0);
        
        test_utils::destroy(admin_cap);
        test_utils::destroy(config);
    }
}
