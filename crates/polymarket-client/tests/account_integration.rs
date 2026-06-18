//! Account data integration tests.
//!
//! ```bash
//! cargo test -p polymarket-client --features secure --test account_integration -- --ignored --nocapture
//! ```

use polymarket_client::{
    Environment, FetchPortfolioValueRequest, ListPositionsRequest, PublicClient,
};

const DEMO_USER: &str = "0x56687bf447db6ffa42ffe2204a05edaa20f55839";

#[tokio::test]
#[ignore = "live API"]
async fn list_positions_for_known_wallet() {
    let client = PublicClient::new(Environment::production());
    let mut paginator = client
        .list_positions(ListPositionsRequest {
            user: DEMO_USER.into(),
            page_size: Some(5),
            ..Default::default()
        })
        .expect("valid request");

    let page = paginator.first_page().await.expect("positions page");
    // Positions may be empty if the demo wallet has closed all holdings.
    let _ = page.items.len();
}

#[tokio::test]
#[ignore = "live API"]
async fn fetch_portfolio_value_for_known_wallet() {
    let client = PublicClient::new(Environment::production());
    let values = client
        .fetch_portfolio_value(FetchPortfolioValueRequest {
            user: DEMO_USER.into(),
            markets: Vec::new(),
        })
        .await
        .expect("portfolio value");

    assert!(!values.is_empty());
    assert!(!values[0].value.is_empty());
}
