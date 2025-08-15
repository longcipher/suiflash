# SuiFlash Router

A comprehensive flash loan router for the Sui blockchain, providing unified access to multiple DeFi protocol flash loan services with extensible architecture.

## Features

- **Multi-Protocol Support**: Unified interface for flash loans across different DeFi protocols
- **Protocol Abstraction**: Extensible adapter pattern for easy integration of new protocols
- **Receipt-Based Settlement**: Secure flash loan execution with proper validation
- **Fee Optimization**: Accurate fee calculation and transparent cost structure
- **On-Chain Registry**: Decentralized protocol adapter management

## Supported Protocols

| Protocol | Fee Rate | Status | Asset Support |
|----------|----------|---------|---------------|
| [Navi Protocol](https://naviprotocol.gitbook.io/navi-protocol-docs/) | 0.06% | ✅ Integrated | SUI, USDC, USDT |
| Bucket Protocol | 0.05% | 🔄 Planned | Multiple |
| Scallop Protocol | 0.09% | 🔄 Planned | Multiple |

## Architecture

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User dApp     │    │  Flash Router   │    │   Protocols     │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │Arbitrage Bot│ │────▶│ flash_loan_ │ │────▶│ Navi Adapter│ │
│ └─────────────┘ │    │ │    coin()   │ │    │ └─────────────┘ │
│ ┌─────────────┐ │    │ └─────────────┘ │    │ ┌─────────────┐ │
│ │Liquidation  │ │    │ ┌─────────────┐ │    │ │Bucket Adapter│ │
│ │   Bot       │ │    │ │Protocol     │ │────▶│ └─────────────┘ │
│ └─────────────┘ │    │ │Abstraction  │ │    │ ┌─────────────┐ │
│                 │    │ └─────────────┘ │    │ │Scallop      │ │
└─────────────────┘    └─────────────────┘    │ │Adapter      │ │
                                              │ └─────────────┘ │
                                              └─────────────────┘
```

## Quick Start

### 1. Installation

```bash
# Clone the repository
git clone https://github.com/longcipher/suiflash
cd suiflash/suiflash-contract/suiflash-router

# Build the contract
sui move build
```

### 2. Basic Usage

```move
use suiflash::flash_router;
use suiflash::protocols;

// Execute a flash loan with SUI
entry fun my_arbitrage(
    cfg: &Config,
    amount: u64,
    ctx: &mut TxContext
) {
    flash_router::flash_loan_coin<SUI>(
        cfg,
        protocols::id_navi(),  // Use Navi protocol
        amount,                // Amount to borrow
        tx_context::sender(ctx),
        vector::empty(),       // Custom payload
        ctx
    );
}
```

### 3. Custom Flash Loan Callback

```move
// Implement your callback logic
public fun flash_callback<T>(
    loan: Coin<T>,
    amount: u64,
    fee: u64,
    payload: vector<u8>,
    ctx: &mut TxContext
): Coin<T> {
    // Your arbitrage/liquidation logic here
    // Must return coins worth amount + fee
    
    loan // Return the repayment
}
```

## Module Overview

### Core Modules

- **`main.move`** - Main flash loan router with public entry points
- **`state.move`** - Configuration management and admin capabilities
- **`errors.move`** - Comprehensive error definitions

### Protocol Layer

- **`protocols.move`** - Protocol abstraction layer with unified dispatch
- **`registry.move`** - On-chain registry for protocol adapter addresses

### Protocol Adapters

- **`integrations/navi_integration.move`** - Navi Protocol flash loan adapter

### Interfaces

- **`interfaces.move`** - Standard interfaces for protocol adapters

## Testing

```bash
# Run all tests
sui move test

# Run specific test module
sui move test --test navi_tests
```

## Integration Guide

See [NAVI_INTEGRATION.md](./NAVI_INTEGRATION.md) for detailed integration guide.

### Adding New Protocols

1. Create adapter module in `sources/integrations/`
2. Implement required interface functions
3. Add protocol ID to `protocols.move`
4. Register adapter in on-chain registry
5. Add tests in `tests/`

Example adapter structure:

```move
module suiflash::new_protocol_integration {
    use sui::coin::Coin;
    
    struct NewProtocolReceipt<phantom T> has store {
        principal: u64,
        fee: u64,
    }
    
    public fun borrow<T>(
        amount: u64,
        ctx: &mut TxContext
    ): (Coin<T>, NewProtocolReceipt<T>) {
        // Implementation
    }
    
    public fun settle<T>(
        receipt: NewProtocolReceipt<T>,
        repay_coin: Coin<T>,
        ctx: &mut TxContext
    ): Coin<T> {
        // Implementation
    }
}
```

## Configuration

### Protocol Fees

| Protocol | Fee (bps) | Description |
|----------|-----------|-------------|
| Navi | 6 | 0.06% treasury fee |
| Bucket | 5 | 0.05% protocol fee |
| Scallop | 9 | 0.09% platform fee |

### Contract Addresses

Update the following addresses for mainnet deployment:

```move
// In navi_integration.move
public fun flash_loan_config_id(): address { @0x... }
public fun protocol_package(): address { @0x... }
public fun sui_pool_id(): address { @0x... }
```

## Development

### Prerequisites

- Sui CLI v1.0+
- Move compiler
- Git

### Build

```bash
sui move build
```

### Format

```bash
sui move fmt
```

### Lint

```bash
sui move test --coverage
```

## Security Considerations

1. **Reentrancy Protection**: All flash loan callbacks are atomic transactions
2. **Fee Validation**: Accurate fee calculation prevents underpayment
3. **Receipt Verification**: Cryptographic receipts ensure proper settlement
4. **Access Control**: Admin-only functions for critical operations
5. **Error Handling**: Comprehensive validation and graceful failure

## Roadmap

- [x] Core flash loan router infrastructure
- [x] Navi Protocol integration
- [x] Protocol abstraction layer
- [x] On-chain registry system
- [ ] Bucket Protocol integration
- [ ] Scallop Protocol integration
- [ ] Enhanced BCS serialization
- [ ] Governance module
- [ ] Cross-protocol arbitrage helpers

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Support

For questions and support:

- GitHub Issues: [Create an issue](https://github.com/longcipher/suiflash/issues)
- Documentation: [NAVI_INTEGRATION.md](./NAVI_INTEGRATION.md)
- Examples: [tests/](./tests/)

## Acknowledgments

- [Navi Protocol](https://naviprotocol.gitbook.io/navi-protocol-docs/) for flash loan infrastructure
- [Sui Foundation](https://sui.io/) for the Move programming language
- Flash loan pattern inspiration from Ethereum DeFi ecosystem
