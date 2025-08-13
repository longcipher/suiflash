module suiflash::errors {
    const E_INVALID_PROTOCOL: u64 = 1;
    const E_AMOUNT_TOO_LOW: u64 = 2;
    const E_INSUFFICIENT_REPAYMENT: u64 = 3;
    const E_PAUSED: u64 = 4;
    const E_FORBIDDEN: u64 = 5;
    const E_UNALLOWED_ASSET: u64 = 6;

    public fun invalid_protocol(): u64 { E_INVALID_PROTOCOL }
    public fun amount_too_low(): u64 { E_AMOUNT_TOO_LOW }
    public fun insufficient_repayment(): u64 { E_INSUFFICIENT_REPAYMENT }
    public fun paused(): u64 { E_PAUSED }
    public fun forbidden(): u64 { E_FORBIDDEN }
    public fun unallowed_asset(): u64 { E_UNALLOWED_ASSET }
}
