//! Account data example (positions + portfolio value).
//!
//! ```bash
//! cargo run --example account --features secure
//! ```
//!
//! Uses a well-known Polymarket trader address for the public read demo.
//! With `POLYMARKET_PRIVATE_KEY` set, also fetches authenticated account data.

use polymarket_client::{
    Environment, FetchPortfolioValueRequest, ListPositionsRequest, PublicClient, SecureClient,
    PRIVATE_KEY_VAR,
};

const DEMO_USER: &str = "0x56687bf447db6ffa42ffe2204a05edaa20f55839";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let client = PublicClient::new(Environment::production());

    println!("Fetching positions for {DEMO_USER}…\n");
    let mut positions = client.list_positions(ListPositionsRequest {
        user: DEMO_USER.into(),
        page_size: Some(5),
        ..Default::default()
    })?;
    let page = positions.first_page().await?;
    for position in &page.items {
        println!(
            "• {} — {} shares @ {} (PnL {})",
            position.title, position.size, position.avg_price, position.cash_pnl
        );
    }

    let values = client
        .fetch_portfolio_value(FetchPortfolioValueRequest {
            user: DEMO_USER.into(),
            markets: Vec::new(),
        })
        .await?;
    if let Some(value) = values.first() {
        println!("\nPortfolio value: {} USDC", value.value);
    }

    if let Ok(private_key) = std::env::var(PRIVATE_KEY_VAR) {
        println!("\nAuthenticated account reads…");
        let secure = SecureClient::builder()
            .environment(Environment::production())
            .private_key(private_key)
            .build()
            .await?;

        let notifications = secure.fetch_notifications().await?;
        println!("Notifications: {}", notifications.len());

        let rewards = secure.list_current_rewards().await?;
        println!("Current reward markets: {}", rewards.len());
    } else {
        println!("\nSet POLYMARKET_PRIVATE_KEY to demo authenticated account endpoints.");
    }

    Ok(())
}
