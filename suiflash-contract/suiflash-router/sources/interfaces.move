#[allow(duplicate_alias)]
module suiflash::interfaces {
    use sui::tx_context::TxContext;
    use sui::coin::Coin;

    /*******************************************************
    * Callback Interface Definitions
    *
    * Legacy simple form (kept for backward compatibility during migration):
    *   public entry fun flash_loan_callback_amount<T>(amount: u64, payload: vector<u8>, ctx: &mut TxContext): u64
    * Returns repayment amount.
    *
    * New coin form (recommended):
    *   public entry fun flash_loan_callback<T>(loan_coin: Coin<T>, payload: vector<u8>, ctx: &mut TxContext): Coin<T>
    * Must return a Coin<T> whose value >= principal + protocol fee + service fee.
    *******************************************************/

    /// Placeholder amount-form invoke (echoes amount)
    public fun invoke_callback_amount(_recipient: address, amount: u64, _payload: vector<u8>, _ctx: &mut TxContext): u64 { amount }

    /// Placeholder coin-form invoke: returns coin unchanged
    public fun invoke_callback_coin<T>(_recipient: address, coin: Coin<T>, _payload: vector<u8>, _ctx: &mut TxContext): Coin<T> { coin }
}
