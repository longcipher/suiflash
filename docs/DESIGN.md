# SuiFlash Design

## 1. Project Overview

**Project Name**: SuiFlash  
**Core Objective**: Create a flash loan aggregator on the Sui blockchain.

### Working Modes

- **Aggregation**: SuiFlash itself does not hold liquidity.
- **Routing**: When receiving user (or bot) flash loan requests, it intelligently selects one of the backend supported protocols (NAVI, Bucket, Scallop) to initiate the actual flash loan.
- **Abstraction**: Provides users with a unified, simplified flash loan interface, so users don't need to care about which specific protocol provides the funds.
- **Execution**: Transfers borrowed funds and execution control to the user's contract.
- **Settlement**: Before the same transaction ends, ensures user repays principal + original protocol interest, and collects a small service fee, finally returning funds to the original lending protocol.

### Technology Stack

- **On-chain Contract**: Sui Move
- **Backend/Bot**: Rust + Artemis Framework

### Core Concept

The core of flash loans lies in **atomicity**: all operations within one transaction either all succeed or all fail. Sui's Programmable Transaction Blocks (PTB) naturally support such complex atomic operations.

SuiFlash's design consists of two main parts:
1. **On-chain Smart Contract** (`sui_flash` module): The core of the project, responsible for executing all flash loan aggregation, routing, and settlement logic on-chain.
2. **Off-chain Artemis Bot** (`suiflash_bot`): A backend service based on the Artemis framework. It monitors on-chain state (such as liquidity and rates of various protocols), receives user requests, builds complex PTBs and submits them to the Sui network. Users can interact directly with the bot, or the bot can autonomously execute strategies like arbitrage.

## 2. Detailed Workflow Example

### Request Initiation
A user (or their MEV bot) calls the `suiflash_bot` API or triggers its strategy, initiating a flash loan request for 1000 SUI. The request includes: loan amount, currency, and a target contract address (`user_contract`) that receives funds and executes logic.

### Backend Processing (Artemis)
1. `suiflash_bot` receives the request.
2. It queries NAVI, Bucket, Scallop protocols for current SUI flash loan rates and available liquidity (Artemis Collector work).
3. Assume Scallop has the lowest rate (0.05%). The bot decides to use Scallop (Artemis Strategy work).
4. The bot starts building a PTB.

### On-Chain Execution (Sui Move)
- **Step A - Call SuiFlash**: The first step of the PTB submitted by the bot calls our core contract: `sui_flash::flash_loan`, with parameters:
  - `amount`: 1000
  - `coin_type`: SUI
  - `protocol_choice`: SCALLOP (enum or u64)
  - `recipient_contract`: user_contract address
  - `callback_payload`: user-defined data to pass to user_contract

- **Step B - Call Underlying Protocol**: The `sui_flash` contract internally calls `scallop::flash_loan` based on `protocol_choice`, requesting 1000 SUI.

- **Step C - Funds and Receipt**: Scallop contract sends 1000 SUI Coin object and a ScallopLoanReceipt object to the `sui_flash` contract.

- **Step D - Callback User Contract**: `sui_flash` contract immediately calls a standard interface function of `user_contract` (e.g., `execute_operation`), passing the 1000 SUI Coin object and `callback_payload`.

- **Step E - User Logic**: `user_contract` receives 1000 SUI and executes arbitrage/liquidation logic. For example, buy X Token with 1000 SUI on DEX A, then sell X Token on DEX B for 1002 SUI.

- **Step F - Return Funds to SuiFlash**: `user_contract` must return funds to the `sui_flash` contract. The returned amount must be at least principal + Scallop fee + SuiFlash fee. In this example, Scallop fee is 1000 * 0.05% = 0.5 SUI. Assume SuiFlash fee is 0.04% = 0.4 SUI. User needs to return at least 1000.9 SUI. User earned 2 SUI, so returns 1001 SUI.

- **Step G - Settlement and Repayment**: `sui_flash` contract receives 1001 SUI. It verifies the amount is sufficient. It separates 1000.5 SUI to return to Scallop. It calls `scallop::repay_flash_loan`, passing 1000.5 SUI and the previously received ScallopLoanReceipt. The remaining 0.5 SUI is SuiFlash profit, transferred to the designated treasury address.

- **Step H - Transaction Complete**: All operations in the PTB execute successfully, transaction is confirmed on-chain. If any step fails (e.g., user fails to return sufficient funds), the entire transaction rolls back as if nothing happened.

### Fee Model
**Total Fee = Underlying Protocol Fee + SuiFlash Service Fee**
- **Underlying Protocol Fee**: Determined by NAVI, Bucket, Scallop respectively (e.g., 0.05%).
- **SuiFlash Service Fee**: Set by SuiFlash contract admin, e.g., 0.04%. This is our profit source.

## 3. Code Architecture

The project is divided into two independent packages:

### 3.1 On-chain Contract: `sui_flash_contract`

A standard Sui Move package:

```
sui_flash_contract/
├── Move.toml         # Package config, dependencies (sui, scallop, navi, bucket)
└── sources/
    ├── main.move         # Core entry function `flash_loan`
    ├── integrations/
    │   ├── scallop.move  # Scallop interaction logic
    │   ├── navi.move     # NAVI interaction logic
    │   └── bucket.move   # Bucket interaction logic
    ├── interfaces.move   # User contract callback interface definitions
    ├── state.move        # Contract state: AdminCap, fee config, etc.
    └── errors.move       # Custom error codes
```

### 3.2 Off-chain Bot: `suiflash_bot`

A standard Rust project using the Artemis framework:

```
suiflash_bot/
├── Cargo.toml        # Dependencies (artemis-core, sui-sdk, tokio, etc.)
├── src/
│   ├── main.rs         # Program entry, initialize and run Artemis Executor
│   ├── collectors.rs   # Collectors: gather protocol rates and liquidity data
│   ├── strategies.rs   # Strategies: decide when and how to initiate flash loans
│   ├── executors.rs    # Executors: build and send Sui PTB
│   └── config.rs       # Config module (RPC, private key, contract addresses)
└── .env              # Environment variables (private key, RPC_URL)
```

## 4. Module Design Details

### 4.1 On-chain Contract (`sui_flash_contract`)

#### state.move
- **SuiFlashAdminCap**: Admin capability object for setting fees, treasury address, etc.
- **Config**: Shared object storing `fee_bps` (service fee basis points) and `treasury` (treasury address).

#### errors.move
Define errors like `EInvalidProtocol`, `EAmountTooLow`, `EInsufficientRepayment`.

#### interfaces.move
Define FlashLoanCallback interface (implemented through function signature convention in Move). User contracts must have a public function like `public entry fun execute_operation<T>(loaned_coin: Coin<T>, payload: vector<u8>, ctx: &mut TxContext)` that receives loaned funds and custom data from our contract.

#### integrations/*.move
Each file contains a module specifically for wrapping calls to a specific protocol:
- `scallop::borrow_and_repay`: Encapsulates calling `scallop_protocol::flash_loan` and `scallop_protocol::repay_flash_loan` logic.
- `navi::borrow_and_repay`: Similarly encapsulates NAVI logic.
- `bucket::borrow_and_repay`: Similarly encapsulates Bucket logic.

This isolates changes and makes `main.move` cleaner.

#### main.move
- **Protocol Constants**: `PROTOCOL_SCALLOP`, `PROTOCOL_NAVI`, `PROTOCOL_BUCKET` for protocol selectors.
- **Entry Function**: `public entry fun flash_loan<CoinType>(...)` receiving parameters: `config: &Config`, `protocol_selector: u64`, `amount: u64`, `recipient: address`, `payload: vector<u8>`, `ctx: &mut TxContext`.
- Uses match statement based on `protocol_selector` to call corresponding `integrations::*` modules.
- Integration modules handle borrowing, user callback, receiving repayment, returning to original protocol.
- After receiving user repayment, calculates and collects SuiFlash service fee, transfers to `config.treasury`.

### 4.2 Off-chain Bot (`suiflash_bot`)

#### config.rs
Define Config struct, load through dotenv or environment variables: Sui RPC URL, bot wallet private key, `sui_flash` contract package ID and Config object ID.

#### collectors.rs
**ProtocolDataCollector**: An Artemis Collector.
- Periodically (or on-demand) reads flash loan rates and pool liquidity from Scallop, NAVI, Bucket contracts through `sui_client.read_api().get_dynamic_fields()` and `sui_client.move_call()` methods.
- Sends this information as Events to the Artemis event stream.

#### strategies.rs
**ArbitrageStrategy**: An Artemis Strategy.
- Listens to data events from ProtocolDataCollector and possible external API request events.
- When arbitrage opportunities are found or user requests received, calculates optimal flash loan scheme (which protocol, how much to borrow).
- Creates an Action containing all parameters needed to execute the flash loan.

#### executors.rs
**SuiFlashLoanExecutor**: An Artemis Executor.
- Listens to Actions from ArbitrageStrategy.
- When receiving an Action:
  - Uses `sui-sdk`'s `ProgrammableTransactionBuilder` to build transactions.
  - `ptb.entry_fun()` calls `sui_flash::flash_loan` with all necessary parameters (including protocol choice, amount, user callback contract address from Action).
  - Signs PTB using bot's `SuiKeyPair`.
  - Sends transaction through `sui_client.quorum_driver_api().execute_transaction_block()`.
  - Records transaction results.

#### main.rs
- Read configuration.
- Initialize SuiClient.
- Create instances of ProtocolDataCollector, ArbitrageStrategy, SuiFlashLoanExecutor.
- Assemble them into an Artemis Engine.
- `engine.run().await` starts the bot.

## 5. REST API Integration

The bot exposes REST endpoints for external users:
- `POST /flashloan` - Submit flash loan request with JSON payload
- `GET /protocols` - Query current protocol rates and liquidity
- `GET /status` - Bot health and statistics

## 6. Security & Risk Management

- **Atomicity**: PTB ensures all-or-nothing execution
- **Fee Validation**: On-chain verification of repayment sufficiency
- **Protocol Isolation**: Each integration module handles protocol-specific logic
- **Admin Controls**: Pausable, fee adjustable via AdminCap
- **Reentrancy Protection**: Loan receipts prevent nested borrowing

## 7. Development Phases

**Phase 1**: Core infrastructure (collectors, basic strategies, Move contracts)
**Phase 2**: REST API, advanced routing strategies, monitoring
**Phase 3**: Multi-protocol splitting, optimization, production hardening

## 8. Current Implementation Status

- ✅ Rust module structure with collector/routing/aggregator
- ✅ Basic cost and liquidity routing strategies
- ✅ Move contract skeleton with modular integrations
- 🚧 Artemis integration (collectors/strategies/executors)
- 🚧 REST API layer
- 🚧 Real protocol integrations (currently using test mints)
- ❌ Production deployment and monitoring

This design provides a comprehensive foundation for a production-ready flash loan aggregator on Sui, with clear separation between on-chain execution and off-chain intelligence.
