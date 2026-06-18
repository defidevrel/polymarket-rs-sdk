//! Secure client integration tests (requires `POLYMARKET_PRIVATE_KEY`).
//!
//! Run with:
//! ```bash
//! POLYMARKET_PRIVATE_KEY=0x… cargo test -p polymarket-client --features secure --test secure_integration -- --ignored --nocapture
//! ```

use polymarket_client::{Environment, ListOpenOrdersRequest, SecureClient, PRIVATE_KEY_VAR};

#[tokio::test]
#[ignore = "live API + private key"]
async fn secure_client_authenticates_and_lists_orders() {
    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("POLYMARKET_PRIVATE_KEY required");

    let secure = SecureClient::builder()
        .environment(Environment::production())
        .private_key(private_key)
        .build()
        .await
        .expect("build secure client");

    assert!(!secure.credentials().key.is_empty());

    secure
        .setup_trading_approvals()
        .await
        .expect("setup trading approvals");

    let _orders = secure
        .list_open_orders(ListOpenOrdersRequest::default())
        .await
        .expect("list open orders");
}
