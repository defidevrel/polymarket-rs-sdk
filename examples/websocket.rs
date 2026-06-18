//! WebSocket subscription example.
//!
//! ```bash
//! cargo run -p polymarket-client --example websocket --features websockets
//! ```

use std::time::Duration;

use futures::StreamExt as _;
use polymarket_client::{
    Environment, ListMarketsRequest, MarketSubscription, PublicClient, SubscriptionSpec,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let client = PublicClient::new(Environment::production());

    let mut markets = client.list_markets(ListMarketsRequest {
        closed: Some(false),
        page_size: Some(1),
        ..Default::default()
    })?;
    let page = markets.first_page().await?;
    let market = page
        .items
        .first()
        .ok_or("no open markets found")?;
    let token_id = market
        .outcomes
        .yes
        .token_id
        .as_ref()
        .ok_or("market missing yes token")?
        .to_string();

    println!("Subscribing to market stream for token {token_id}…\n");

    let mut handle = client
        .subscribe(vec![SubscriptionSpec::Market(MarketSubscription {
            token_ids: vec![token_id],
            custom_feature_enabled: false,
        })])?;

    match tokio::time::timeout(Duration::from_secs(30), handle.next()).await {
        Ok(Some(Ok(event))) => println!("{event:?}"),
        Ok(Some(Err(error))) => return Err(error.into()),
        Ok(None) => eprintln!("websocket stream closed before event"),
        Err(_) => eprintln!("timed out waiting for websocket event"),
    }

    handle.close();
    Ok(())
}
