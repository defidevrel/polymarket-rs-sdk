# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

### Added (unreleased)

- crates.io publish metadata; examples moved into `crates/polymarket-client/examples/`
- Hybrid architecture docs and `hybrid_server` example — HTTP adapter for Solana (or any) front-end → Polymarket settlement on Polygon

## [0.1.2] - 2026-06-21

### Changed

- README and crate docs no longer reference the TypeScript SDK

## [0.1.1] - 2026-06-21

### Fixed

- docs.rs builds with `all-features` so `SecureClient`, websockets, and account APIs appear in the online docs

## [0.1.0] - 2026-06-18

### Added

- Workspace crates: `polymarket-types`, `polymarket-bindings`, `polymarket-client`
- `PublicClient` for discovery and market data
  - `list_markets`, `fetch_market`, `list_events`
  - `fetch_midpoint`, `fetch_order_book`
  - Keyset pagination with cursor resume
- `SecureClient` (`secure` feature) for authenticated trading
  - Builder with private key / API credentials
  - `setup_trading_approvals`, limit and market orders
  - Order fetch, cancel, and list open orders
- Account data (`account` / `secure` features)
  - `list_positions`, `fetch_portfolio_value`, `list_activity`
  - `list_account_trades`, `fetch_notifications`, rewards and scoring reads
  - On-chain CTF: `split_position`, `merge_positions`, `redeem_positions`
- Realtime streams (`websockets` / `secure` features)
  - Unified `subscribe()` with market, user, sports, RTDS channels
  - `SubscriptionHandle` with idempotent `close()`
- Typed errors per action with guard helpers
- Examples: `quickstart`, `trading`, `account`, `websocket`, `hybrid_server`
- Live integration tests (opt-in via `#[ignore]`)
- CI: fmt, clippy, unit tests, doc build

### Not yet supported

- RFQ / combos websocket and REST surface
- Equity price RTDS subscriptions
- `transfer_erc20` (relayer)
- Full discovery parity (`search`, `list_tags`, `list_teams`, comments REST)

[0.1.0]: https://github.com/your-org/polymarket-rs-sdk/releases/tag/v0.1.0
