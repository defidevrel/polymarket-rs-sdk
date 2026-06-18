//! WebSocket client wiring and subscription routing.

#![allow(clippy::unnecessary_wraps)]

mod types;

pub use types::{
    CommentsSubscription, CryptoPricesSubscription, EquityPricesSubscription, MarketStreamEvent,
    MarketSubscription, SportsStreamEvent, StreamEvent, SubscribeError, SubscriptionHandle,
    SubscriptionSpec, UserStreamEvent, UserSubscription,
};

pub use types::merge_streams;

use std::str::FromStr as _;

use futures::StreamExt as _;
use polymarket_client_sdk_v2::clob::ws::types::response::WsMessage;
use polymarket_client_sdk_v2::clob::ws::Client as ClobWsClient;
use polymarket_client_sdk_v2::rtds::Client as RtdsClient;
use polymarket_client_sdk_v2::types::{B256, U256};
use polymarket_client_sdk_v2::ws::config::Config as WsConfig;

use crate::environment::Environment;
use crate::error::user_input;
use crate::public_client::PublicClient;
use types::{EventStream, RtdsStreamEvent};

#[derive(Clone)]
pub struct WebSocketClients {
    pub clob: ClobWsClient,
    pub rtds: RtdsClient,
    #[cfg(feature = "secure")]
    pub clob_ws_url: String,
    pub sports_url: String,
}

impl WebSocketClients {
    pub fn new(environment: &Environment) -> Result<Self, SubscribeError> {
        ensure_rustls_crypto_provider();
        Ok(Self {
            clob: ClobWsClient::new(environment.clob_ws, WsConfig::default())
                .map_err(|e| SubscribeError::Transport(e.to_string()))?,
            rtds: RtdsClient::new(environment.rtds_ws, WsConfig::default())
                .map_err(|e| SubscribeError::Transport(e.to_string()))?,
            #[cfg(feature = "secure")]
            clob_ws_url: environment.clob_ws.to_string(),
            sports_url: environment.sports_ws.to_string(),
        })
    }
}

impl PublicClient {
    /// Subscribe to one or more realtime channels.
    pub fn subscribe(
        &self,
        specs: Vec<SubscriptionSpec>,
    ) -> Result<SubscriptionHandle, SubscribeError> {
        if specs.is_empty() {
            return Err(SubscribeError::UserInput(user_input(
                "at least one subscription spec is required",
            )));
        }

        let ws = &self.ws;

        let mut streams = Vec::with_capacity(specs.len());
        for spec in specs {
            streams.push(subscribe_one(ws, spec)?);
        }

        Ok(SubscriptionHandle::new(merge_streams(streams), None))
    }
}

pub fn subscribe_one(
    ws: &WebSocketClients,
    spec: SubscriptionSpec,
) -> Result<EventStream, SubscribeError> {
    match spec {
        SubscriptionSpec::Market(market) => subscribe_market(ws, market),
        SubscriptionSpec::Sports => subscribe_sports(ws),
        SubscriptionSpec::Comments(comments) => subscribe_comments(ws, comments),
        SubscriptionSpec::CryptoPricesBinance(crypto) => subscribe_crypto_binance(ws, crypto),
        SubscriptionSpec::CryptoPricesChainlink(crypto) => subscribe_crypto_chainlink(ws, crypto),
        SubscriptionSpec::EquityPrices(equity) => subscribe_equity(ws, equity),
        SubscriptionSpec::User(_) => Err(SubscribeError::UserInput(user_input(
            "user subscriptions require SecureClient",
        ))),
    }
}

fn subscribe_market(
    ws: &WebSocketClients,
    spec: MarketSubscription,
) -> Result<EventStream, SubscribeError> {
    if spec.token_ids.is_empty() {
        return Err(SubscribeError::UserInput(user_input(
            "market subscription requires at least one token_id",
        )));
    }

    let asset_ids = parse_token_ids(&spec.token_ids)?;
    let clob = ws.clob.clone();

    let book = clob
        .subscribe_orderbook(asset_ids.clone())
        .map_err(map_ws_err)?;
    let prices = clob
        .subscribe_prices(asset_ids.clone())
        .map_err(map_ws_err)?;
    let trades = clob
        .subscribe_last_trade_price(asset_ids.clone())
        .map_err(map_ws_err)?;

    let mut streams: Vec<EventStream> = vec![
        Box::pin(book.map(map_market_book)),
        Box::pin(prices.map(map_market_price)),
        Box::pin(trades.map(map_market_last_trade)),
    ];

    if spec.custom_feature_enabled {
        let bba = clob.subscribe_best_bid_ask(asset_ids).map_err(map_ws_err)?;
        streams.push(Box::pin(bba.map(map_market_bba)));
    }

    Ok(merge_streams(streams))
}

fn subscribe_sports(ws: &WebSocketClients) -> Result<EventStream, SubscribeError> {
    let url = ws.sports_url.clone();
    Ok(Box::pin(async_stream::try_stream! {
        use futures::SinkExt as _;
        use tokio_tungstenite::connect_async;
        use tokio_tungstenite::tungstenite::Message;

        let (mut socket, _) = connect_async(&url)
            .await
            .map_err(|e| SubscribeError::Transport(e.to_string()))?;

        while let Some(message) = socket.next().await {
            let message = message.map_err(|e| SubscribeError::Transport(e.to_string()))?;
            match message {
                Message::Text(text) => {
                    if text == "ping" {
                        socket
                            .send(Message::Text("pong".into()))
                            .await
                            .map_err(|e| SubscribeError::Transport(e.to_string()))?;
                        continue;
                    }
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(event) = parse_sports_event(&value) {
                            yield StreamEvent::Sports(event);
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    }))
}

fn subscribe_comments(
    ws: &WebSocketClients,
    _spec: CommentsSubscription,
) -> Result<EventStream, SubscribeError> {
    let rtds = ws.rtds.clone();
    Ok(Box::pin(async_stream::try_stream! {
        let subscribed = match rtds.subscribe_comments(None) {
            Ok(stream) => stream,
            Err(error) => Err(map_ws_err(error))?,
        };
        let mut stream = std::pin::pin!(subscribed);
        while let Some(result) = stream.next().await {
            match result {
                Ok(comment) => {
                    yield StreamEvent::Rtds(RtdsStreamEvent::Comment {
                        entity_id: comment.parent_entity_id.to_string(),
                        body: comment.body,
                    });
                }
                Err(error) => Err(map_ws_err(error))?,
            }
        }
    }))
}

fn subscribe_crypto_binance(
    ws: &WebSocketClients,
    spec: CryptoPricesSubscription,
) -> Result<EventStream, SubscribeError> {
    let rtds = ws.rtds.clone();
    let symbols = if spec.symbols.is_empty() {
        None
    } else {
        Some(spec.symbols)
    };
    Ok(Box::pin(async_stream::try_stream! {
        let subscribed = match rtds.subscribe_crypto_prices(symbols) {
            Ok(stream) => stream,
            Err(error) => Err(map_ws_err(error))?,
        };
        let mut stream = std::pin::pin!(subscribed);
        while let Some(result) = stream.next().await {
            match result {
                Ok(price) => {
                    yield StreamEvent::Rtds(RtdsStreamEvent::CryptoPrice {
                        source: "binance".into(),
                        symbol: price.symbol,
                        price: price.value.to_string(),
                    });
                }
                Err(error) => Err(map_ws_err(error))?,
            }
        }
    }))
}

fn subscribe_crypto_chainlink(
    ws: &WebSocketClients,
    spec: CryptoPricesSubscription,
) -> Result<EventStream, SubscribeError> {
    let rtds = ws.rtds.clone();
    if spec.symbols.len() <= 1 {
        let symbol = spec.symbols.into_iter().next();
        return Ok(Box::pin(async_stream::try_stream! {
            let subscribed = match rtds.subscribe_chainlink_prices(symbol) {
                Ok(stream) => stream,
                Err(error) => Err(map_ws_err(error))?,
            };
            let mut stream = std::pin::pin!(subscribed);
            while let Some(result) = stream.next().await {
                match result {
                    Ok(price) => {
                        yield StreamEvent::Rtds(RtdsStreamEvent::CryptoPrice {
                            source: "chainlink".into(),
                            symbol: price.symbol,
                            price: price.value.to_string(),
                        });
                    }
                    Err(error) => Err(map_ws_err(error))?,
                }
            }
        }));
    }

    let mut streams: Vec<EventStream> = Vec::with_capacity(spec.symbols.len());
    for symbol in spec.symbols {
        let rtds = ws.rtds.clone();
        streams.push(Box::pin(async_stream::try_stream! {
            let subscribed = match rtds.subscribe_chainlink_prices(Some(symbol)) {
                Ok(stream) => stream,
                Err(error) => Err(map_ws_err(error))?,
            };
            let mut stream = std::pin::pin!(subscribed);
            while let Some(result) = stream.next().await {
                match result {
                    Ok(price) => {
                        yield StreamEvent::Rtds(RtdsStreamEvent::CryptoPrice {
                            source: "chainlink".into(),
                            symbol: price.symbol,
                            price: price.value.to_string(),
                        });
                    }
                    Err(error) => Err(map_ws_err(error))?,
                }
            }
        }));
    }
    Ok(merge_streams(streams))
}

fn subscribe_equity(
    _ws: &WebSocketClients,
    _spec: EquityPricesSubscription,
) -> Result<EventStream, SubscribeError> {
    Err(SubscribeError::Transport(
        "equity price subscriptions are not yet supported in the Rust SDK".into(),
    ))
}

#[cfg(feature = "secure")]
pub fn subscribe_user(
    ws: &WebSocketClients,
    credentials: polymarket_client_sdk_v2::auth::Credentials,
    address: polymarket_client_sdk_v2::types::Address,
    spec: UserSubscription,
) -> Result<EventStream, SubscribeError> {
    let markets = if spec.markets.is_empty() {
        Vec::new()
    } else {
        spec.markets
            .iter()
            .map(|market| {
                B256::from_str(market).map_err(|e| {
                    SubscribeError::UserInput(user_input(format!("invalid market: {e}")))
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    let clob = ClobWsClient::new(&ws.clob_ws_url, WsConfig::default()).map_err(map_ws_err)?;
    let auth = clob
        .authenticate(credentials, address)
        .map_err(map_ws_err)?;

    let stream = auth.subscribe_user_events(markets).map_err(map_ws_err)?;

    Ok(Box::pin(stream.filter_map(|result| async move {
        match result {
            Ok(message) => map_user_message(message).map(Ok),
            Err(error) => Some(Err(map_ws_err(error))),
        }
    })))
}

fn parse_token_ids(token_ids: &[String]) -> Result<Vec<U256>, SubscribeError> {
    token_ids
        .iter()
        .map(|token_id| {
            U256::from_str(token_id).map_err(|e| {
                SubscribeError::UserInput(user_input(format!("invalid token_id: {e}")))
            })
        })
        .collect()
}

fn map_ws_err(error: impl std::fmt::Display) -> SubscribeError {
    SubscribeError::Transport(error.to_string())
}

fn map_market_book(
    result: Result<
        polymarket_client_sdk_v2::clob::ws::types::response::BookUpdate,
        impl std::fmt::Display,
    >,
) -> Result<StreamEvent, SubscribeError> {
    let book = result.map_err(map_ws_err)?;
    Ok(StreamEvent::Market(MarketStreamEvent::OrderBook {
        token_id: book.asset_id.to_string(),
        market: book.market.to_string(),
        timestamp: book.timestamp,
        bid_levels: book.bids.len(),
        ask_levels: book.asks.len(),
    }))
}

fn map_market_price(
    result: Result<
        polymarket_client_sdk_v2::clob::ws::types::response::PriceChange,
        impl std::fmt::Display,
    >,
) -> Result<StreamEvent, SubscribeError> {
    let change = result.map_err(map_ws_err)?;
    let entry =
        change.price_changes.into_iter().next().ok_or_else(|| {
            SubscribeError::Transport("price change event missing entries".into())
        })?;
    Ok(StreamEvent::Market(MarketStreamEvent::PriceChange {
        token_id: entry.asset_id.to_string(),
        market: change.market.to_string(),
        price: entry.price.to_string(),
        side: format!("{:?}", entry.side),
    }))
}

fn map_market_last_trade(
    result: Result<
        polymarket_client_sdk_v2::clob::ws::types::response::LastTradePrice,
        impl std::fmt::Display,
    >,
) -> Result<StreamEvent, SubscribeError> {
    let trade = result.map_err(map_ws_err)?;
    Ok(StreamEvent::Market(MarketStreamEvent::LastTradePrice {
        token_id: trade.asset_id.to_string(),
        market: trade.market.to_string(),
        price: trade.price.to_string(),
    }))
}

fn map_market_bba(
    result: Result<
        polymarket_client_sdk_v2::clob::ws::types::response::BestBidAsk,
        impl std::fmt::Display,
    >,
) -> Result<StreamEvent, SubscribeError> {
    let bba = result.map_err(map_ws_err)?;
    Ok(StreamEvent::Market(MarketStreamEvent::BestBidAsk {
        token_id: bba.asset_id.to_string(),
        best_bid: bba.best_bid.to_string(),
        best_ask: bba.best_ask.to_string(),
    }))
}

#[cfg(feature = "secure")]
fn map_user_message(message: WsMessage) -> Option<StreamEvent> {
    match message {
        WsMessage::Order(order) => Some(StreamEvent::User(UserStreamEvent::Order {
            order_id: order.id,
            token_id: order.asset_id.to_string(),
            side: format!("{:?}", order.side),
            status: order
                .status
                .map_or_else(|| "UNKNOWN".into(), |status| format!("{status:?}")),
        })),
        WsMessage::Trade(trade) => Some(StreamEvent::User(UserStreamEvent::Trade {
            trade_id: trade.id,
            token_id: trade.asset_id.to_string(),
            side: format!("{:?}", trade.side),
            price: trade.price.to_string(),
            size: trade.size.to_string(),
        })),
        _ => None,
    }
}

fn parse_sports_event(value: &serde_json::Value) -> Option<SportsStreamEvent> {
    Some(SportsStreamEvent {
        game_id: value.get("gameId")?.as_i64()?,
        league: value
            .get("leagueAbbreviation")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        status: value
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        score: value
            .get("score")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        live: value
            .get("live")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    })
}

fn ensure_rustls_crypto_provider() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}
