module suiflash::errors {
    /// Error constants with #[error] attribute for better error reporting
    
    #[error]
    const E_INVALID_PROTOCOL: u64 = 1;
    #[error]
    const E_AMOUNT_TOO_LOW: u64 = 2;
    #[error]
    const E_INSUFFICIENT_REPAYMENT: u64 = 3;
    #[error]
    const E_PAUSED: u64 = 4;
    #[error]
    const E_FORBIDDEN: u64 = 5;
    #[error]
    const E_UNALLOWED_ASSET: u64 = 6;
    #[error]
    const E_INDEX_OUT_OF_BOUNDS: u64 = 7;
    #[error]
    const E_INVALID_FEE_BPS: u64 = 8;
    #[error]
    const E_ASSET_TYPE_MISMATCH: u64 = 9;

    /// Public error accessor functions using named constants
    public fun invalid_protocol(): u64 { E_INVALID_PROTOCOL }
    public fun amount_too_low(): u64 { E_AMOUNT_TOO_LOW }
    public fun insufficient_repayment(): u64 { E_INSUFFICIENT_REPAYMENT }
    public fun paused(): u64 { E_PAUSED }
    public fun forbidden(): u64 { E_FORBIDDEN }
    public fun unallowed_asset(): u64 { E_UNALLOWED_ASSET }
    public fun index_out_of_bounds(): u64 { E_INDEX_OUT_OF_BOUNDS }
    public fun invalid_fee_bps(): u64 { E_INVALID_FEE_BPS }
    public fun asset_type_mismatch(): u64 { E_ASSET_TYPE_MISMATCH }
}
