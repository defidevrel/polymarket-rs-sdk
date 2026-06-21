use polymarket_client::{
    Environment, FetchMidpointRequest, FetchOrderBookRequest, ListMarketsRequest, PublicClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let client = PublicClient::new(Environment::production());

    println!("Fetching open markets…\n");

    let mut markets = client.list_markets(ListMarketsRequest {
        closed: Some(false),
        page_size: Some(5),
        ..Default::default()
    })?;

    let page = markets.first_page().await?;
    for market in &page.items {
        let question = market.question.as_deref().unwrap_or("(no question)");
        println!("• {question}");
        println!("  id: {}", market.id);
        if let Some(yes) = market.outcomes.yes.token_id.as_ref() {
            println!("  yes token: {yes}");
            if let Ok(mid) = client
                .fetch_midpoint(FetchMidpointRequest {
                    token_id: yes.as_str().to_string(),
                })
                .await
            {
                println!("  midpoint: {mid}");
            }
            if let Ok(book) = client
                .fetch_order_book(FetchOrderBookRequest {
                    token_id: yes.as_str().to_string(),
                })
                .await
            {
                println!(
                    "  order book: {} bids, {} asks",
                    book.bids.len(),
                    book.asks.len()
                );
            }
        }
        println!();
    }

    Ok(())
}
