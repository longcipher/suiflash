#[allow(duplicate_alias)]
module suiflash::registry {
    //! Simplified ProtocolRegistry (sequential append model)
    //! Each appended adapter gets the next numeric protocol_id (index in entries).
    //! Governing principle: no sparse insertion, no removal (future upgrade can
    //! append new version and deprecate old off‑chain).

    use std::vector;
    use sui::object;
    use sui::tx_context::TxContext;
    use sui::transfer;
    use suiflash::state::AdminCap;

    public struct ProtocolRegistry has key, store { id: UID, entries: vector<address> }

    entry fun init_and_share_registry(ctx: &mut TxContext) {
        let reg = ProtocolRegistry { id: object::new(ctx), entries: vector::empty<address>() };
        transfer::share_object(reg);
    }

    /// Append a new adapter (returns assigned id implicit via index length-1)
    entry fun append_adapter(_cap: &AdminCap, reg: &mut ProtocolRegistry, adapter_pkg: address) {
        vector::push_back(&mut reg.entries, adapter_pkg);
    }

    /// Update existing adapter in-place (governance controlled)
    entry fun update_adapter(_cap: &AdminCap, reg: &mut ProtocolRegistry, protocol_id: u64, new_adapter_pkg: address) {
        let len = vector::length(&reg.entries);
    if (protocol_id >= len) { abort 0 };
        *vector::borrow_mut(&mut reg.entries, protocol_id) = new_adapter_pkg;
    }

    public fun adapter(reg: &ProtocolRegistry, protocol_id: u64): address {
        let len = vector::length(&reg.entries);
    if (protocol_id >= len) { abort 0 }; // out-of-range
        *vector::borrow(&reg.entries, protocol_id)
    }

    public fun count(reg: &ProtocolRegistry): u64 { vector::length(&reg.entries) }
}
