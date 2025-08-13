# SuiFlash Bot

A Rust-based flash loan aggregator bot for the Sui blockchain, built using the Artemis framework. This bot provides intelligent routing across multiple flash loan protocols (NAVI, Bucket, Scallop) and exposes a REST API for easy integration.

## Features

- **Multi-Protocol Support**: Aggregates flash loans from NAVI, Bucket, and Scallop protocols
- **Intelligent Routing**: Automatic best-cost or highest-liquidity routing strategies
- **Real-time Data Collection**: Continuous monitoring of protocol fees and liquidity
- **REST API**: Easy-to-use HTTP endpoints for flash loan requests
- **Artemis Integration**: Built on the proven Artemis framework for MEV extraction
- **Configurable**: Environment-based configuration for different networks and strategies

## Architecture

The bot consists of three main components following the Artemis pattern:

1. **Collectors** (`src/collectors.rs`): Gather real-time data from flash loan protocols
2. **Strategies** (`src/strategies.rs`): Implement routing logic and execution planning
3. **Executors** (`src/executors.rs`): Execute flash loan transactions on-chain

## Quick Start

### Prerequisites

- Rust 1.70+
- Access to a Sui RPC endpoint
- Private key for transaction signing

### Installation

1. Clone the repository:
```bash
git clone https://github.com/your-org/sui-flash
cd sui-flash/suiflash_bot
```

2. Copy environment configuration:
```bash
cp .env.example .env
```

3. Edit `.env` with your configuration:
```bash
# Required
PRIVATE_KEY=your_sui_private_key_here
SUI_FLASH_PACKAGE_ID=0x1234567890abcdef
SUI_FLASH_CONFIG_OBJECT_ID=0xabcdef1234567890

# Optional
SUI_RPC_URL=https://fullnode.testnet.sui.io:443
SERVER_PORT=3000
STRATEGY=cheapest
REFRESH_INTERVAL_MS=10000
```

4. Build and run:
```bash
cargo build --release
cargo run
```

## API Usage

### Flash Loan Request

**POST** `/flashloan`

```json
{
  "asset": "SUI",
  "amount": 1000000000,
  "route_mode": "BestCost",
  "explicit_protocol": null,
  "user_operation": "arbitrage_operation"
}
```

**Response:**
```json
{
  "transaction_digest": "0x1234567890abcdef...",
  "protocol_used": "Navi",
  "protocol_fee": 800000,
  "service_fee": 0,
  "total_fee": 800000
}
```

### Route Modes

- `BestCost`: Selects the protocol with the lowest fees
- `BestLiquidity`: Selects the protocol with the highest available liquidity
- `Explicit`: Uses the protocol specified in `explicit_protocol`

### Health Check

**GET** `/health`

Returns `OK` if the service is running.

## Configuration

All configuration is done via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `SUI_RPC_URL` | Sui RPC endpoint | `https://fullnode.testnet.sui.io:443` |
| `PRIVATE_KEY` | Private key for signing transactions | Required |
| `SUI_FLASH_PACKAGE_ID` | Deployed flash loan contract package ID | Required |
| `SUI_FLASH_CONFIG_OBJECT_ID` | Configuration object ID | Required |
| `SERVER_PORT` | HTTP server port | `3000` |
| `STRATEGY` | Default routing strategy | `cheapest` |
| `REFRESH_INTERVAL_MS` | Data collection interval | `10000` |
| `CONTRACT_PACKAGE_ID` | Main contract package ID | `0x1` |
| `NAVI_PACKAGE_ID` | NAVI protocol package ID | `0x2` |
| `BUCKET_PACKAGE_ID` | Bucket protocol package ID | `0x3` |
| `SCALLOP_PACKAGE_ID` | Scallop protocol package ID | `0x4` |

## Development

### Running Tests

```bash
cargo test
```

### Running with Debug Logs

```bash
RUST_LOG=debug cargo run
```

### Project Structure

```
src/
├── main.rs              # HTTP server and main application
├── config.rs            # Configuration and data types
├── collectors.rs        # Protocol data collection
├── strategies.rs        # Routing strategies and execution planning
└── executors.rs         # Transaction execution
```

## Protocol Integration

The bot currently supports three major Sui flash loan protocols:

1. **NAVI Protocol**: Decentralized lending protocol
2. **Bucket Protocol**: Stablecoin and lending platform
3. **Scallop Protocol**: Yield farming and lending

Each protocol has different fee structures and liquidity characteristics, which the bot monitors continuously to provide optimal routing.

## Security Considerations

- Private keys are managed through environment variables
- All transactions are signed locally before submission
- The bot includes transaction verification and error handling
- Rate limiting and monitoring should be implemented for production use

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For questions and support:
- Open an issue on GitHub
- Join our Discord community
- Check the documentation at [docs.suiflash.io](https://docs.suiflash.io)
