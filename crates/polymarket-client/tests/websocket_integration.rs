//! WebSocket integration tests.
//!
//! ```bash
//! cargo test -p polymarket-client --features websockets --test websocket_integration -- --ignored --nocapture
//! ```

use std::time::Duration;

use futures::StreamExt as _;
use polymarket_client::{
    Environment, ListMarketsRequest, MarketSubscription, PublicClient, StreamEvent,
    SubscriptionSpec,
};

async fn active_token_id(client: &PublicClient) -> String {
    let mut markets = client
        .list_markets(ListMarketsRequest {
            closed: Some(false),
            page_size: Some(5),
            ..Default::default()
        })
        .expect("valid request");
    let page = markets.first_page().await.expect("markets page");
    for market in &page.items {
        if let Some(token_id) = market.outcomes.yes.token_id.as_ref() {
            return token_id.to_string();
        }
    }
    panic!("no token id found in open markets");
}

#[tokio::test]
#[ignore = "live websocket"]
async fn market_channel_receives_event() {
    let client = PublicClient::new(Environment::production());
    let token_id = active_token_id(&client).await;

    let mut handle = client
        .subscribe(vec![SubscriptionSpec::Market(MarketSubscription {
            token_ids: vec![token_id],
            custom_feature_enabled: false,
        })])
        .expect("subscribe");

    let result = tokio::time::timeout(Duration::from_secs(45), async {
        while let Some(item) = handle.next().await {
            match item {
                Ok(StreamEvent::Market(_)) => return,
                Ok(_) => {}
                Err(error) => panic!("websocket error: {error}"),
            }
        }
        panic!("stream closed before market event");
    })
    .await;

    handle.close();

    assert!(result.is_ok(), "timed out waiting for market websocket event");
}
