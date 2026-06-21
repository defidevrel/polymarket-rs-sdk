# polymarket-client

Unified [Polymarket](https://polymarket.com) Rust SDK — discovery, market data, trading, account reads, and websockets.

Settlement is on **Polygon** via Polymarket's CLOB and CTF contracts.

## Install

```toml
[dependencies]
polymarket-client = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

**Trading + websockets:**

```toml
polymarket-client = { version = "0.1", features = ["secure"] }
```

## Quickstart

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
        println!("{}", market.question.as_deref().unwrap_or(""));
    }
    Ok(())
}
```

## Features

| Feature | Enables |
|---------|---------|
| *(default)* | HTTP discovery + CLOB market data |
| `account` | Data API reads on `PublicClient` |
| `websockets` | Realtime `subscribe()` |
| `secure` | `account` + `websockets` + `SecureClient` trading and CTF |

## Examples

```bash
cargo run --example quickstart
cargo run --example hybrid_server --features secure
```

Full documentation: [GitHub](https://github.com/defidevrel/polymarket-rs-sdk) · [docs.rs](https://docs.rs/polymarket-client/latest/polymarket_client/)
