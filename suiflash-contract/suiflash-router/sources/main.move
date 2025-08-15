#[allow(duplicate_alias, lint(abort_without_constant))]
module suiflash::main {
    use sui::tx_context::TxContext;
    use sui::event;
    use suiflash::errors;
    use suiflash::state::{Config, service_fee_bps, assert_not_paused};
    use suiflash::protocols::{protocol_fee_bps, borrow_with_receipt, settle_with_receipt};
    use suiflash::interfaces::invoke_callback_amount as invoke_callback;

    // (legacy constants removed; use re-exports at bottom sourced from protocols module)

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
    entry fun flash_loan(cfg: &Config, protocol: u64, amount: u64, recipient: address, payload: vector<u8>, ctx: &mut TxContext) {
        assert_not_paused(cfg);
        if (amount == 0) { abort errors::amount_too_low() };

    let proto_fee_bps = protocol_fee_bps(protocol);
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
    /// Now supports Navi-style receipt-based flash loans with proper settlement.
    entry fun flash_loan_coin<CoinType>(cfg: &Config, protocol: u64, amount: u64, recipient: address, payload: vector<u8>, ctx: &mut TxContext) {
        use sui::transfer;
        use sui::coin;
        use suiflash::interfaces::invoke_callback_coin;

        assert_not_paused(cfg);
        if (amount == 0) { abort errors::amount_too_low() };
        let proto_fee_bps = protocol_fee_bps(protocol);
        let proto_fee = amount * proto_fee_bps / 10_000;
        let svc_fee_bps = service_fee_bps(cfg);
        let svc_fee = amount * svc_fee_bps / 10_000;
        let total_required = amount + proto_fee + svc_fee;

        // Borrow via protocol adapter with receipt for proper settlement
        let (loan, receipt_bytes) = borrow_with_receipt<CoinType>(protocol, amount, ctx);

        // Invoke user callback with borrowed funds
        let returned_coin = invoke_callback_coin(recipient, loan, payload, ctx);

        // Settle the flash loan using protocol-specific logic and receipt
        // Note: create a zero coin as placeholder for loan since it was consumed by callback
        let placeholder_loan = coin::zero<CoinType>(ctx);
        let settled = settle_with_receipt<CoinType>(protocol, placeholder_loan, receipt_bytes, returned_coin, ctx);

        // Verify sufficient repayment (for protocols that don't enforce internally)
        let returned_amount = coin::value(&settled);
        if (returned_amount < total_required) { abort errors::insufficient_repayment() };

        let ev = FlashLoanEvent { 
            protocol, 
            amount, 
            protocol_fee: proto_fee, 
            service_fee: svc_fee, 
            total_repayment: returned_amount 
        };
        event::emit(ev);
        
        // Transfer final settlement to recipient
        transfer::public_transfer(settled, recipient);
    }    // Re‑export constants removed to prevent duplicate symbol warnings; use suiflash::protocols::PROTOCOL_* instead.
}
