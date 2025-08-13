#[allow(duplicate_alias)]
module suiflash::state {
    use std::vector;
    use sui::tx_context::TxContext;
    use sui::object;

    /// Admin capability (transferable by design, restrict with policy if needed).
    public struct AdminCap has key, store { id: UID }

    /// Config shared object storing fee and treasury address plus pause flag & allowed assets.
    public struct Config has key, store {
        id: UID,
        treasury: address,
        service_fee_bps: u64,
        paused: bool,
        allowed_assets: vector<u64>, // placeholder asset tags
    }

    public fun create(treasury: address, service_fee_bps: u64, ctx: &mut TxContext): (AdminCap, Config) {
        assert!(service_fee_bps <= 10_000, 0);
        (AdminCap { id: object::new(ctx) }, Config { id: object::new(ctx), treasury, service_fee_bps, paused: false, allowed_assets: vector::empty() })
    }

    public fun assert_not_paused(cfg: &Config) { assert!(!cfg.paused, 0); }

    public fun set_paused(cap: &AdminCap, cfg: &mut Config, value: bool) { cfg.paused = value; let _ = cap; }

    public fun set_service_fee(cap: &AdminCap, cfg: &mut Config, fee_bps: u64) { assert!(fee_bps <= 10_000, 0); cfg.service_fee_bps = fee_bps; let _ = cap; }

    public fun service_fee_bps(cfg: &Config): u64 { cfg.service_fee_bps }

    public fun add_allowed_asset(cap: &AdminCap, cfg: &mut Config, tag: u64) { vector::push_back(&mut cfg.allowed_assets, tag); let _ = cap; }

    public fun is_allowed_asset(cfg: &Config, tag: u64): bool { contains(&cfg.allowed_assets, tag, 0) }

    fun contains(v: &vector<u64>, tag: u64, i: u64): bool {
        if (i >= vector::length(v)) { false }
        else if (*vector::borrow(v, i) == tag) { true }
        else { contains(v, tag, i + 1) }
    }
}
