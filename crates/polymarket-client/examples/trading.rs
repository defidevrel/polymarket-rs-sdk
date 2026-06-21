//! Authenticated trading example (requires `secure` feature + private key).
//!
//! ```bash
//! POLYMARKET_PRIVATE_KEY=0x… cargo run --example trading --features secure
//! ```
//!
//! By default this lists open orders and does **not** place live orders.
//! Set `POLYMARKET_PLACE_ORDER=1` to place a small limit buy (demo only).

use polymarket_client::{
    Environment, ListMarketsRequest, ListOpenOrdersRequest, OrderSide, PlaceLimitOrderRequest,
    SecureClient, PRIVATE_KEY_VAR,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let private_key =
        std::env::var(PRIVATE_KEY_VAR).expect("set POLYMARKET_PRIVATE_KEY to your wallet key");

    let secure = SecureClient::builder()
        .environment(Environment::production())
        .private_key(private_key)
        .build()
        .await?;

    println!("Authenticated wallet: {}", secure.wallet());
    println!("API key: {}", secure.credentials().key);

    secure.setup_trading_approvals().await?;
    println!("Trading approvals synced with CLOB.\n");

    let open = secure
        .list_open_orders(ListOpenOrdersRequest::default())
        .await?;
    println!("Open orders: {}", open.len());
    for order in open.iter().take(5) {
        println!(
            "  {} {:?} {} @ {} ({})",
            order.order_id, order.side, order.original_size, order.price, order.status
        );
    }

    if std::env::var("POLYMARKET_PLACE_ORDER").as_deref() == Ok("1") {
        let token_id = sample_yes_token_id(&secure).await?;
        println!("\nPlacing demo limit buy on token {token_id}…");

        let response = secure
            .place_limit_order(PlaceLimitOrderRequest {
                token_id,
                side: OrderSide::Buy,
                price: 0.01,
                size: 5.0,
                expiration: None,
                post_only: true,
            })
            .await?;

        println!(
            "Order result: ok={} id={:?} message={:?}",
            response.ok, response.order_id, response.message
        );
    } else {
        println!("\nSkipping order placement (set POLYMARKET_PLACE_ORDER=1 to enable).");
    }

    Ok(())
}

async fn sample_yes_token_id(client: &SecureClient) -> Result<String, Box<dyn std::error::Error>> {
    let mut markets = client.list_markets(ListMarketsRequest {
        closed: Some(false),
        page_size: Some(10),
        ..Default::default()
    })?;
    let page = markets.first_page().await?;
    let market = page.items.first().ok_or("no open markets found")?;
    market
        .outcomes
        .yes
        .token_id
        .as_ref()
        .map(|id| id.as_str().to_string())
        .ok_or_else(|| "market has no yes token".into())
}
