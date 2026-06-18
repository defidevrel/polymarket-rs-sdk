# Polymarket Rust SDK

Unified Rust SDK for [Polymarket](https://polymarket.com), modeled after the official TypeScript SDK ([`@polymarket/client`](https://github.com/Polymarket/ts-sdk)).

Built for production use: typed errors, input validation, HTTPS-only transport (`rustls`), request timeouts, and normalized domain models matching the TS SDK.

## Quickstart

Add to your `Cargo.toml`:

```toml
polymarket-client = { path = "crates/polymarket-client" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use polymarket_client::{Environment, ListMarketsRequest, PublicClient};

#[tokio::main]
async fn main() -> Result<(), polymarket_client::Error> {
    let client = PublicClient::new(Environment::production());

    let mut markets = client.list_markets(ListMarketsRequest {
        closed: Some(false),
        page_size: Some(5),
        ..Default::default()
    })?;

    let page = markets.first_page().await?;
    for market in &page.items {
        println!(
            "{} — {}",
            market.id,
            market.question.as_deref().unwrap_or("")
        );
    }

    Ok(())
}
```

Run the included example:

```bash
cargo run --example quickstart
```

## Secure client (trading)

Enable the `secure` feature to authenticate and trade via the official CLOB SDK (`polymarket_client_sdk_v2`):

```toml
polymarket-client = { path = "crates/polymarket-client", features = ["secure"] }
```

```rust
use polymarket_client::{Environment, OrderSide, PlaceLimitOrderRequest, SecureClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secure = SecureClient::builder()
        .environment(Environment::production())
        .private_key(std::env::var("POLYMARKET_PRIVATE_KEY")?)
        .build()
        .await?;

    secure.setup_trading_approvals().await?;

    let response = secure.place_limit_order(PlaceLimitOrderRequest {
        token_id: "…".into(),
        side: OrderSide::Buy,
        price: 0.50,
        size: 10.0,
        expiration: None,
        post_only: false,
    }).await?;

    println!("order placed: {:?}", response.order_id);
    Ok(())
}
```

Trading example (lists open orders; set `POLYMARKET_PLACE_ORDER=1` to place a demo order):

```bash
POLYMARKET_PRIVATE_KEY=0x… cargo run --example trading --features secure
```

Secure integration tests:

```bash
POLYMARKET_PRIVATE_KEY=0x… cargo test -p polymarket-client --features secure --test secure_integration -- --ignored --nocapture
```

Account data (public read, no key required):

```bash
cargo run --example account --features secure
cargo test -p polymarket-client --features secure --test account_integration -- --ignored --nocapture
```

## Websockets

Enable the `websockets` feature (included in `secure`):

```toml
polymarket-client = { path = "crates/polymarket-client", features = ["websockets"] }
```

```rust
use futures::StreamExt as _;
use polymarket_client::{
    Environment, MarketSubscription, PublicClient, SubscriptionSpec,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PublicClient::new(Environment::production());

    let mut handle = client.subscribe(vec![SubscriptionSpec::Market(MarketSubscription {
        token_ids: vec!["…".into()],
        custom_feature_enabled: false,
    })])?;

    if let Some(Ok(event)) = handle.next().await {
        println!("{event:?}");
    }
    handle.close();
    Ok(())
}
```

```bash
cargo run --example websocket --features websockets
cargo test -p polymarket-client --features websockets --test websocket_integration -- --ignored --nocapture
```

User-channel subscriptions require `SecureClient::subscribe` with `SubscriptionSpec::User`.

## Features

| Area | Methods | Status |
|------|---------|--------|
| Discovery | `list_markets`, `fetch_market`, `list_events` | ✅ |
| Market data | `fetch_midpoint`, `fetch_order_book` | ✅ |
| Account data | `list_positions`, `fetch_portfolio_value`, `list_activity` | ✅ (`account` / `secure`) |
| Trading / auth | `SecureClient`, orders, notifications, rewards, CTF | ✅ (`secure` feature) |
| Realtime | `subscribe`, market/user/RTDS/sports streams | ✅ (`websockets` / `secure`) |

## Architecture

Three crates mirror the [TypeScript monorepo](https://github.com/Polymarket/ts-sdk):

- **`polymarket-types`** — Branded IDs (`MarketId`, `TokenId`), addresses, validation
- **`polymarket-bindings`** — API deserialization + normalization (Gamma → `Market`, CLOB → `OrderBook`)
- **`polymarket-client`** — `PublicClient`, `SecureClient` (feature-gated), pagination, HTTP layer

## Error handling

Each action returns typed errors with guards matching the TS SDK:

```rust
use polymarket_client::{ListMarketsError, PublicClient};

match client.list_markets(request) {
    Ok(mut paginator) => { /* … */ }
    Err(e) if ListMarketsError::is_error(&e) => { /* handle SDK error */ }
    Err(e) => return Err(e.into()),
}
```

## Testing

Unit tests (offline):

```bash
cargo test
```

Live integration tests (hit production APIs):

```bash
cargo test -p polymarket-client --test integration -- --ignored --nocapture
```

## Security

See [SECURITY.md](SECURITY.md). Summary:

- Private keys stay in your environment — never commit `POLYMARKET_PRIVATE_KEY`
- Trading uses `polymarket_client_sdk_v2` for EIP-712 / HMAC auth and order signing
- HTTPS-only HTTP client with TLS via `rustls`
- 30s request timeout, sanitized error bodies (no HTML dumps)
- Input validation at every public API boundary

## License

MIT — same as the [Polymarket TypeScript SDK](https://github.com/Polymarket/ts-sdk).

## Related

- [Polymarket TypeScript SDK](https://github.com/Polymarket/ts-sdk)
- [Polymarket docs](https://docs.polymarket.com/dev-tooling/typescript)
- [Official Rust CLOB client](https://github.com/Polymarket/rs-clob-client-v2) (CLOB-only; this SDK targets the unified surface)
- [CHANGELOG.md](CHANGELOG.md)

## API documentation

Generate local docs (includes `secure` and `websockets` APIs):

```bash
cargo doc -p polymarket-client --features secure --no-deps --open
```
