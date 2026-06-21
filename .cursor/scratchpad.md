# Polymarket Rust SDK (Unified Client)

## Background and Motivation

Build a **unified Rust SDK** for Polymarket modeled after the beta TypeScript SDK (`@polymarket/client` from [Polymarket/ts-sdk](https://github.com/Polymarket/ts-sdk)), not the legacy CLOB-only client surface.

**Why this matters:** The TS unified SDK exposes one consistent API across:
- Discovery (events, markets, tags, teams, search, comments, sports)
- Market data (order books, prices, history, batch reads)
- Realtime streams (market/user/RTDS/sports/RFQ websockets)
- Authenticated trading (orders, positions, portfolio, activity, wallet ops, CTF split/merge/redeem)

**Existing landscape:**
| Package | Scope | Repo |
|---------|-------|------|
| `@polymarket/client@beta` | Unified (target model) | github.com/Polymarket/ts-sdk |
| `polymarket_client_sdk_v2` | CLOB + optional gamma/data/ws features | github.com/Polymarket/rs-clob-client-v2 |
| `@polymarket/clob-client-v2` | CLOB-only (older unified layer) | github.com/Polymarket/clob-client-v2 |

**Workspace:** `/Users/test/rust-sdk` is empty (greenfield).

**Target API parity (Rust idioms):**
```rust
// Public client
let client = PublicClient::new(Environment::production());
let markets = client.list_markets(ListMarketsParams { closed: false, page_size: 5 }).await?;
let first = markets.first_page().await?;
for page in markets.pages().await? { /* ... */ }

// Secure client
let secure = SecureClient::builder()
    .signer(local_signer)
    .build()
    .await?;
secure.setup_trading_approvals().await?;
let resp = secure.place_limit_order(PlaceLimitOrderParams { /* ... */ }).await?;
```

---

## Key Challenges and Analysis

### 1. Scope is large — phased delivery required
The TS SDK has ~116 source files in `packages/client` alone, plus `packages/types` and `packages/bindings`. Full parity includes wallet deployment, relayer, EIP-712 auth, HMAC L2, on-chain CTF ops, and multi-socket streaming.

**Recommendation:** Ship in phases; each phase is independently useful and testable.

### 2. Reuse vs. greenfield
Official `polymarket_client_sdk_v2` already implements:
- CLOB REST + auth (L1/L2)
- Gamma/data/bridge modules (feature-gated)
- WebSocket (`ws`, `rtds` features)
- Alloy-based signing

**Recommendation:** Wrap/extend `polymarket_client_sdk_v2` as an internal dependency for low-level HTTP/auth rather than reimplementing CLOB signing from scratch. Build the **unified surface** (normalized types, pagination, error guards, client composition) in this repo.

### 3. Rust API design choices
| TS pattern | Rust equivalent |
|------------|-----------------|
| `createPublicClient()` | `PublicClient::new()` / builder |
| Paginator with `for await` | `Paginator<T>` implementing `Stream<Item = Result<Page<T>>>` |
| Error guards (`ListMarketsError.isError`) | `enum ListMarketsError` + `impl From` / `is_*` helpers |
| Branded types (`MarketId`, `TokenId`) | newtype structs with `Display`, `FromStr`, serde |
| `createSecureClient` async setup | `SecureClient::builder().build().await?` |
| Wallet adapters (viem/privy/ethers) | trait `Signer` + `alloy` signer impl (phase 1) |

### 4. Crate layout (proposed workspace)
```
polymarket-rs-sdk/
├── Cargo.toml                 # workspace
├── crates/
│   ├── polymarket-types/      # branded primitives, Market, Event, etc. (mirror @polymarket/types)
│   ├── polymarket-bindings/   # raw API DTOs + normalization (mirror @polymarket/bindings)
│   ├── polymarket-client/     # PublicClient, SecureClient, actions, pagination, errors
│   └── polymarket-client/examples/
```

### 5. Dependencies (initial)
- `reqwest` — HTTP
- `serde` / `serde_json` — serialization
- `tokio` — async runtime
- `futures` / `async-stream` — pagination streams
- `thiserror` — typed errors
- `alloy` — EVM signing (secure client)
- `polymarket_client_sdk_v2` — optional internal bridge for CLOB (evaluate during Phase 3)
- `tokio-tungstenite` — websockets (Phase 5)

---

## High-level Task Breakdown

### Phase 0 — Project bootstrap
**Tasks:**
1. Init Cargo workspace with `polymarket-types`, `polymarket-bindings`, `polymarket-client` crates
2. CI basics: `cargo fmt`, `clippy`, `test`
3. README with quickstart mirroring TS docs

**Success criteria:** `cargo build` and `cargo test` pass; empty `PublicClient::new()` compiles.

---

### Phase 1 — Public client: discovery + normalized types
**Tasks:**
1. Define branded types: `MarketId`, `EventId`, `TokenId`, `EvmAddress`, `DecimalString`, `IsoDateTime`
2. Define normalized `Market`, `Event` structs (match TS shape)
3. Implement `Environment` config (production URLs for gamma, clob, data APIs)
4. Implement HTTP layer + `ServiceClient` (base URL, retries, rate limit detection)
5. Implement pagination: `Paginator<T>`, `Page<T>`, cursor resume via `from_cursor()`
6. Discovery actions:
   - `list_markets`, `list_events`, `list_tags`, `list_teams`, `search`
   - `fetch_market` (by id/slug/url), `fetch_event`
7. Typed errors per action with `RateLimitError`, `UserInputError`, etc.

**Success criteria:** Integration test fetches live markets (`closed: false`, `page_size: 5`) and parses into `Market`.

---

### Phase 2 — PublicKey data (public)
**Tasks:**
1. `fetch_order_book`, `fetch_price`, `fetch_midpoint`, `fetch_spread`, `fetch_last_trade_price`
2. `fetch_price_history`, batch price/midpoint endpoints
3. `fetch_market_tags`, `fetch_event_tags`
4. `list_sports`, comments endpoints (lower priority within phase)

**Success criteria:** Given a live market token ID, fetch order book + midpoint successfully.

---

### Phase 3 — Secure client: auth + trading
**Tasks:**
1. `Signer` trait + alloy local signer
2. L1 EIP-712 API key create/derive; credential persistence types
3. L2 HMAC request signing
4. `SecureClient::builder()` with deposit wallet resolution (or explicit wallet)
5. `setup_trading_approvals()`
6. Order lifecycle: `create_limit_order`, `post_order`, `place_limit_order`, `place_market_order`
7. Order management: `fetch_order`, `list_open_orders`, `cancel_order`, `cancel_market_orders`

**Success criteria:** Testnet or dry-run path validates order **creation/signing**; live order placement behind explicit env flag.

---

### Phase 4 — Account data + positions
**Tasks:**
1. `list_positions`, `fetch_portfolio_value`, `list_activity`, `list_account_trades`
2. `fetch_notifications`
3. Rewards/scoring: `list_current_rewards`, `fetch_order_scoring`
4. CTF: `split_position`, `merge_positions`, `redeem_positions`
5. Wallet ops: `transfer_erc20`

**Success criteria:** Authenticated read of positions for a known wallet (read-only test).

---

### Phase 5 — Realtime streams
**Tasks:**
1. Unified `subscribe(specs)` returning `Stream<Item = StreamEvent>`
2. Route to market / user / RTDS / sports / RFQ sockets
3. `StreamHandle::close()`

**Success criteria:** Subscribe to market channel for one token; receive at least one book or price event.

---

### Phase 6 — Polish & parity
**Tasks:**
1. Combos/RFQ if needed
2. Examples mirroring TS docs (one per major section)
3. Docs on docs.rs
4. Changelog aligned with TS SDK releases

---

## Project Status Board

- [x] Phase 0 — Project bootstrap
- [x] Phase 1 — Public client discovery
- [x] Phase 2 — Market data
- [x] Phase 3 — Secure client + trading (feature-gated `secure`)
- [x] Phase 4 — Account data + CTF (positions, portfolio, activity, notifications, rewards, split/merge/redeem)
- [x] Phase 5 — Websockets
- [x] Phase 6 — Polish

## Current Status / Progress Tracking

**Phase 6 complete (2026-06-18):** CHANGELOG, expanded crate docs (`lib.rs` feature matrix), per-crate README stubs, CI hardened (secure clippy, doc build, example matrix). RFQ/combos deferred.

**Mintlify docs (2026-06-21):** Live at https://polymarket-rs.mintlify.app. Source repo: [defidevrel/polymarket-rs-docs](https://github.com/defidevrel/polymarket-rs-docs) (`polymarket-sdk/*.mdx`, `docs.json`).

**Phase 5 complete (2026-06-18):** `websockets` feature with unified `subscribe(specs) -> SubscriptionHandle` on `PublicClient` and `SecureClient` (user channel). Routes market (CLOB WS), sports (raw ping/pong), RTDS (comments, crypto binance/chainlink), and authenticated user events. Example: `examples/websocket.rs`. Live test: `tests/websocket_integration.rs` (passes).

**Phase 4 complete (2026-06-18):** Data API account reads on `PublicClient` (`list_positions`, `fetch_portfolio_value`, `list_activity`). `SecureClient` adds wallet-default reads, `list_account_trades`, `fetch_notifications`, `fetch_order_scoring`, `list_current_rewards`, and on-chain CTF ops (`split_position`, `merge_positions`, `redeem_positions`). Example: `examples/account.rs`. Tests: `tests/account_integration.rs`.

## Executor's Feedback or Assistance Requests

All planned phases (0–6) implemented. Optional follow-ups: RFQ REST/WS, discovery parity (`search`, tags, teams), `transfer_erc20`, docs.rs publish when crate is public on crates.io.

## Lessons

- Polymarket docs index: https://docs.polymarket.com/llms.txt
- Official unified API reference (external): https://github.com/Polymarket/ts-sdk
- Official Rust CLOB SDK: https://github.com/Polymarket/rs-clob-client-v2 (CLOB-only; not unified surface)
- Binary market filter is binding-layer behavior — list skips multi-outcome, single fetch fails validation
- No HTTP retries in core ServiceClient — don't add unless mirroring gasless later
- Integration tests hit live APIs — no response mocking by default
- WebSocket TLS requires a rustls crypto provider — `WebSocketClients::new` installs `ring` via `ensure_rustls_crypto_provider()`
- **Detailed docs (Mintlify):** https://polymarket-rs.mintlify.app — edit [defidevrel/polymarket-rs-docs](https://github.com/defidevrel/polymarket-rs-docs). API reference stays on docs.rs.
