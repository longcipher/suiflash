# AI Assistant Project Instructions (SuiFlash)

Purpose: Capital-light multi-protocol flash loan aggregator on Sui. Two halves:
- On-chain Move package: routing + settlement (`suiflash-contract/suiflash-router`)
- Off-chain Rust bot (`suiflash-bot`): data collection, routing strategy, PTB execution, REST API

## 1. Architecture Mental Model
Flow: Client -> Axum REST (`main.rs`) -> Strategy (select protocol) -> Executor (build PTB) -> Move router `flash_loan` -> user callback -> repay + fees -> event.
Move modules: `router.move` (entry), `protocols.move` (dispatch indices), `integrations/*` (protocol adapters), `state.move` (config/admin), `errors.move`, `interfaces.move`.
Rust modules: `collectors.rs` (periodic protocol data snapshot), `strategies.rs` (plan selection), `executors.rs` (PTB & execution), `config.rs` (layered config + API types), `main.rs` (wiring + handlers).
Protocol enum order (Rust): Navi=0, Bucket=1, Scallop=2 (must match on-chain selector indices; keep stable if adding new).

## 2. Configuration Pattern
Central entry: `Config::load()` (do NOT hand-roll env parsing). Priority: SUIFLASH_* env > legacy env > config.toml > defaults.
Add new config field: update `Config` struct + defaults in `load()` + example `config.example.toml` + README table.
Avoid breaking legacy env names unless deprecating with clear fallback.

## 3. Coding Conventions (Rust)
- Formatting: `taplo fmt` + `cargo +nightly fmt` (CI enforces).
- Lint: Clippy with `-D warnings`, deny `unwrap_used`, `uninlined_format_args`; avoid introducing unwrap/panic. Use `eyre::Result` for fallible paths.
- Large numeric literals: use `_` separators (existing tests follow this).
- Logging: use `tracing::{info,error,warn}`; keep structured context; avoid `println!`.
- Fee math: cast to `u128` for intermediate, convert back with checked/try conversions (see service fee calc in `handle_flash_loan`).
- Keep dependency surface minimal (cargo-machete runs in CI). Remove unused crates before committing.

## 4. Coding Conventions (Move)
- Keep selector indices synchronized with Rust `Protocol` enum.
- Errors: add new error codes in `errors.move` and map them in Rust if surfaced.
- State mutations guarded by AdminCap; maintain consistent config fields (service_fee_bps etc.).
- If adding a protocol: new integration module, extend `protocols.move` dispatch & router logic; update README & Rust enum.

## 5. Extending: Add New Protocol (Summary)
1. Move: create `sources/integrations/<protocol>.move`; update `protocols.move` (selector + fee surface); adjust router if needed.
2. Rust: extend `Protocol` enum (append at end), collectors (fetch + store `ProtocolData`), strategy cost/liquidity selection branches, executor mapping.
3. Tests: add unit tests (strategy pick), API test for new protocol enumeration.
4. Docs: update both READMEs tables & configuration if new IDs needed.

## 6. REST & Data Types
Request type `FlashLoanRequest` (config.rs) fields: `asset`, `amount`, `route_mode` (Explicit|BestCost|BestLiquidity), optional `explicit_protocol`, `user_operation`, optional callback fields.
Response types: `FlashLoanResponse`, `ProtocolsResponse`, `StatusResponse`.
Stick to these structs; reuse serialization via `serde`.

## 7. Build & Test Workflows
Local commands:
- Lint: `just lint` (taplo, fmt check, clippy, cargo-machete)
- Format: `just format`
- Tests: `just test` or `cargo test`
- Move build: `sui move build` (inside `suiflash-contract/suiflash-router`)
CI (`.github/workflows/ci.yml`): Rust (fmt, clippy, machete, tests) then Move build/tests.

## 8. Error Handling & Stability
Return `StatusCode::BAD_REQUEST` for strategy/plan user errors, `INTERNAL_SERVER_ERROR` for execution issues. Log root causes with `error!`.
Avoid panics; propagate via `eyre` or map to HTTP status in handlers.

## 9. Performance & Background Tasks
Collectors spawn background refresh (`start_background_collection`). When modifying, ensure graceful task spawning and no blocking in hot request paths.

## 10. Adding Config-driven Behavior
Always: add default via `.set_default`, expose env override automatically. Document new key in config example + README root section.

## 11. Common Pitfalls
- Forgetting to update both Rust & Move when changing protocol indices.
- Introducing an unused dependency (CI will fail at `cargo machete`).
- Using `unwrap` (blocked by lint) or direct env reads instead of `Config::load()`.
- Mismatched service fee computation logic (must mirror on-chain config semantics).

## 12. Quick Reference
Key files: `suiflash-bot/src/{main,config,collectors,strategies,executors}.rs`, Move: `suiflash-contract/suiflash-router/sources/*`.
Run sequence in `main.rs`: load config -> init collector/strategy/executor -> spawn background collection -> start Axum server.

Provide diffs only for changed sections; preserve style & deny warnings.
