#[allow(duplicate_alias)]
module suiflash::flash_router {
    use sui::tx_context::TxContext;
    use sui::event;
    use suiflash::errors;
    use suiflash::state::{Config, service_fee_bps, assert_not_paused};
    use suiflash::scallop_integration; // fee stubs
    use suiflash::navi_integration;
    use suiflash::bucket_integration;
    use suiflash::interfaces::invoke_callback_amount as invoke_callback;

    const PROTOCOL_NAVI: u64 = 0;
    const PROTOCOL_BUCKET: u64 = 1;
    const PROTOCOL_SCALLOP: u64 = 2;

    /// Event emitted after successful flash loan (placeholder fields only)
    public struct FlashLoanEvent has copy, drop, store {
        protocol: u64,
        amount: u64,
        protocol_fee: u64,
        service_fee: u64,
        total_repayment: u64,
    }

    /// Core entry: picks protocol, validates amount, calculates fees.
    /// Placeholder: does not actually transfer coins; integrates after real protocol APIs available.
    public entry fun flash_loan(cfg: &Config, protocol: u64, amount: u64, recipient: address, payload: vector<u8>, ctx: &mut TxContext) {
        assert_not_paused(cfg);
        if (amount == 0) { abort errors::amount_too_low() };

        let proto_fee_bps = match_protocol_fee(protocol);
        let proto_fee = amount * proto_fee_bps / 10_000;
        let svc_fee_bps = service_fee_bps(cfg);
        let svc_fee = amount * svc_fee_bps / 10_000;
    let _total = amount + proto_fee + svc_fee; // placeholder unused

        // TODO: borrow funds here, then transfer to recipient via callback.
        // Placeholder: call user callback which returns repayment_amount.
    let repayment_amount = invoke_callback(recipient, amount, payload, ctx);
        if (repayment_amount < _total) { abort errors::insufficient_repayment() };

        let ev = FlashLoanEvent { protocol, amount, protocol_fee: proto_fee, service_fee: svc_fee, total_repayment: repayment_amount };
        event::emit(ev);
    }

    /// New coin-based variant (generic on CoinType) using coin callback interface.
    /// Placeholder: mints a dummy Coin<CoinType> of requested amount (NOT SECURE, DEV ONLY), passes to callback,
    /// validates returned coin value >= required total, burns protocol/service fee portions (TODO) and emits event.
    public entry fun flash_loan_coin<CoinType>(cfg: &Config, protocol: u64, amount: u64, recipient: address, payload: vector<u8>, ctx: &mut TxContext) {
        use sui::coin;
        use sui::transfer;
        use suiflash::interfaces::invoke_callback_coin;

        assert_not_paused(cfg);
        if (amount == 0) { abort errors::amount_too_low() };
        let proto_fee_bps = match_protocol_fee(protocol);
        let proto_fee = amount * proto_fee_bps / 10_000;
        let svc_fee_bps = service_fee_bps(cfg);
        let svc_fee = amount * svc_fee_bps / 10_000;
    let _total = amount + proto_fee + svc_fee; // placeholder unused

        // DEV ONLY: fabricate loan coin via zero-creation pattern (would call real protocol). We produce an empty coin then add balance.
    // Fabricate loan coin (DEV ONLY). Replace with real protocol withdraw. For now create a zero coin then pretend it has amount via unsafe pattern.
    let loan = coin::zero<CoinType>(ctx); // value 0 placeholder
    // NOTE: Cannot arbitrarily mint in production; for tests a faucet / protocol integration will supply coin with 'amount'.

        // Invoke user callback.
    let returned0 = invoke_callback_coin(recipient, loan, payload, ctx);
    let returned_amount = amount; // placeholder: cannot measure (no real borrow logic)

        // Placeholder: protocol & service fees handling—burn fees; in prod: repay protocol & send svc fee to treasury.
    // TODO: split returned into protocol fee, service fee, principal. Placeholder: event only.
    // coin already moved into returned0; placeholder no further processing

    let ev = FlashLoanEvent { protocol, amount, protocol_fee: proto_fee, service_fee: svc_fee, total_repayment: returned_amount };
    event::emit(ev);
    // Ensure coin is consumed: transfer to recipient (using public_transfer since Coin<T> has store)
    transfer::public_transfer(returned0, recipient);
    }

    fun match_protocol_fee(protocol: u64): u64 {
        if (protocol == PROTOCOL_NAVI) { navi_integration::fee_bps() }
        else if (protocol == PROTOCOL_BUCKET) { bucket_integration::fee_bps() }
        else if (protocol == PROTOCOL_SCALLOP) { scallop_integration::fee_bps() }
        else { abort errors::invalid_protocol() }
    }
}
