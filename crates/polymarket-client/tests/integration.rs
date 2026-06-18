//! Integration tests against live Polymarket APIs.
//!
//! Run with: `cargo test -p polymarket-client --test integration -- --ignored --nocapture`

use polymarket_client::{
    Environment, FetchMidpointRequest, FetchOrderBookRequest, ListEventsRequest,
    ListMarketsRequest, PublicClient,
};

fn client() -> PublicClient {
    PublicClient::new(Environment::production())
}

#[tokio::test]
#[ignore = "live API"]
async fn list_markets_returns_binary_markets() {
    let client = client();
    let mut paginator = client
        .list_markets(ListMarketsRequest {
            closed: Some(false),
            page_size: Some(5),
            ..Default::default()
        })
        .expect("valid request");

    let page = paginator.first_page().await.expect("list markets");
    assert!(!page.items.is_empty(), "expected at least one market");
    for market in &page.items {
        assert!(!market.id.as_str().is_empty());
        assert!(!market.outcomes.yes.label.is_empty());
    }
}

#[tokio::test]
#[ignore = "live API"]
async fn list_events_returns_events() {
    let client = client();
    let mut paginator = client
        .list_events(ListEventsRequest {
            closed: Some(false),
            page_size: Some(5),
            ..Default::default()
        })
        .expect("valid request");

    let page = paginator.first_page().await.expect("list events");
    assert!(!page.items.is_empty(), "expected at least one event");
}

#[tokio::test]
#[ignore = "live API"]
async fn fetch_order_book_and_midpoint() {
    let client = client();
    let mut paginator = client
        .list_markets(ListMarketsRequest {
            closed: Some(false),
            page_size: Some(1),
            ..Default::default()
        })
        .expect("valid request");

    let page = paginator.first_page().await.expect("list markets");
    let market = page.items.first().expect("market");
    let token_id = market
        .outcomes
        .yes
        .token_id
        .as_ref()
        .expect("yes token id")
        .as_str()
        .to_string();

    let midpoint = client
        .fetch_midpoint(FetchMidpointRequest {
            token_id: token_id.clone(),
        })
        .await
        .expect("midpoint");
    assert!(!midpoint.is_empty());

    let book = client
        .fetch_order_book(FetchOrderBookRequest { token_id })
        .await
        .expect("order book");
    assert_eq!(
        book.token_id.as_str(),
        market.outcomes.yes.token_id.as_ref().unwrap().as_str()
    );
}
