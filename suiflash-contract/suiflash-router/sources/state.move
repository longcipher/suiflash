#[allow(duplicate_alias, lint(abort_without_constant))]
module suiflash::state {
    use std::vector;
    use sui::tx_context::TxContext;
    use sui::object;
    use suiflash::errors;

    /// Admin capability (transferable by design, restrict with policy if needed).
    public struct AdminCap has key, store { id: UID }

    /// Config shared object storing fee and treasury address plus pause flag & allowed assets.
    public struct Config has key, store {
        id: UID,
        treasury: address,
        service_fee_bps: u64,
        paused: bool,
        allowed_assets: vector<u64>, // placeholder asset tags
        protocol_configs: vector<address>, // index == protocol ID, value == external protocol config object address (if any)
    }

    public fun create(treasury: address, service_fee_bps: u64, ctx: &mut TxContext): (AdminCap, Config) {
        assert!(service_fee_bps <= 10_000, errors::invalid_fee_bps());
        (AdminCap { id: object::new(ctx) }, Config { id: object::new(ctx), treasury, service_fee_bps, paused: false, allowed_assets: vector::empty(), protocol_configs: vector::empty() })
    }

    public fun assert_not_paused(cfg: &Config) { assert!(!cfg.paused, errors::paused()); }

    public fun set_paused(cap: &AdminCap, cfg: &mut Config, value: bool) { cfg.paused = value; let _ = cap; }

    public fun set_service_fee(cap: &AdminCap, cfg: &mut Config, fee_bps: u64) { assert!(fee_bps <= 10_000, errors::invalid_fee_bps()); cfg.service_fee_bps = fee_bps; let _ = cap; }

    public fun service_fee_bps(cfg: &Config): u64 { cfg.service_fee_bps }

    public fun add_allowed_asset(cap: &AdminCap, cfg: &mut Config, tag: u64) { vector::push_back(&mut cfg.allowed_assets, tag); let _ = cap; }

    /// Store external protocol config object address at index == protocol id. Grows vector with 0x0 placeholders if needed.
    public fun set_protocol_config(cap: &AdminCap, cfg: &mut Config, protocol_id: u64, config_addr: address) {
        let _ = cap;
        grow_if_needed(&mut cfg.protocol_configs, protocol_id);
        *vector::borrow_mut(&mut cfg.protocol_configs, protocol_id) = config_addr;
    }

    public fun protocol_config(cfg: &Config, protocol_id: u64): address {
        if (protocol_id >= vector::length(&cfg.protocol_configs)) { @0x0 } else { *vector::borrow(&cfg.protocol_configs, protocol_id) }
    }

    public fun is_allowed_asset(cfg: &Config, tag: u64): bool { contains(&cfg.allowed_assets, tag, 0) }

    fun contains(v: &vector<u64>, tag: u64, i: u64): bool {
        if (i >= vector::length(v)) { false }
        else if (*vector::borrow(v, i) == tag) { true }
        else { contains(v, tag, i + 1) }
    }

    fun grow_if_needed(v: &mut vector<address>, index: u64) {
        use std::vector;
        let len = vector::length(v);
        if (index < len) return;
        
        // Grow vector to required size
        while (vector::length(v) <= index) { 
            vector::push_back(v, @0x0); 
        }
    }
}
