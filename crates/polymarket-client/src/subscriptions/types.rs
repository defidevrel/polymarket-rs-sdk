//! Realtime subscription types and stream events.

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::Stream;

#[derive(Debug, thiserror::Error, Clone)]
pub enum SubscribeError {
    #[error(transparent)]
    UserInput(#[from] crate::error::UserInputError),
    #[error("websocket error: {0}")]
    Transport(String),
}

/// Subscription spec for unified realtime channels.
#[derive(Clone, Debug)]
pub enum SubscriptionSpec {
    Market(MarketSubscription),
    User(UserSubscription),
    Sports,
    Comments(CommentsSubscription),
    CryptoPricesBinance(CryptoPricesSubscription),
    CryptoPricesChainlink(CryptoPricesSubscription),
    EquityPrices(EquityPricesSubscription),
}

#[derive(Clone, Debug)]
pub struct MarketSubscription {
    pub token_ids: Vec<String>,
    pub custom_feature_enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UserSubscription {
    pub markets: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CommentsSubscription {
    pub parent_entity_id: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct CryptoPricesSubscription {
    pub symbols: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct EquityPricesSubscription {
    pub symbol: String,
}

/// Unified realtime event envelope.
#[derive(Clone, Debug)]
pub enum StreamEvent {
    Market(MarketStreamEvent),
    User(UserStreamEvent),
    Sports(SportsStreamEvent),
    Rtds(RtdsStreamEvent),
}

#[derive(Clone, Debug)]
pub enum MarketStreamEvent {
    OrderBook {
        token_id: String,
        market: String,
        timestamp: i64,
        bid_levels: usize,
        ask_levels: usize,
    },
    PriceChange {
        token_id: String,
        market: String,
        price: String,
        side: String,
    },
    LastTradePrice {
        token_id: String,
        market: String,
        price: String,
    },
    BestBidAsk {
        token_id: String,
        best_bid: String,
        best_ask: String,
    },
}

#[derive(Clone, Debug)]
pub enum UserStreamEvent {
    Order {
        order_id: String,
        token_id: String,
        side: String,
        status: String,
    },
    Trade {
        trade_id: String,
        token_id: String,
        side: String,
        price: String,
        size: String,
    },
}

#[derive(Clone, Debug)]
pub struct SportsStreamEvent {
    pub game_id: i64,
    pub league: String,
    pub status: String,
    pub score: String,
    pub live: bool,
}

#[derive(Clone, Debug)]
pub enum RtdsStreamEvent {
    Comment {
        entity_id: String,
        body: String,
    },
    CryptoPrice {
        source: String,
        symbol: String,
        price: String,
    },
}

pub(super) type EventStream =
    Pin<Box<dyn Stream<Item = Result<StreamEvent, SubscribeError>> + Send>>;

/// Handle for an active subscription (or merged subscriptions).
pub struct SubscriptionHandle {
    stream: EventStream,
    closed: Arc<AtomicBool>,
    on_close: Option<Box<dyn FnOnce() + Send>>,
}

impl SubscriptionHandle {
    pub(crate) fn new(stream: EventStream, on_close: Option<Box<dyn FnOnce() + Send>>) -> Self {
        Self {
            stream,
            closed: Arc::new(AtomicBool::new(false)),
            on_close,
        }
    }

    /// Closes the subscription. Idempotent.
    pub fn close(&mut self) {
        if self.closed.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Some(on_close) = self.on_close.take() {
            on_close();
        }
    }

    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}

impl Stream for SubscriptionHandle {
    type Item = Result<StreamEvent, SubscribeError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.closed.load(Ordering::SeqCst) {
            return Poll::Ready(None);
        }
        Pin::new(&mut self.stream).poll_next(cx)
    }
}

impl Drop for SubscriptionHandle {
    fn drop(&mut self) {
        self.close();
    }
}

pub fn merge_streams(mut streams: Vec<EventStream>) -> EventStream {
    if streams.len() == 1 {
        return streams.pop().expect("non-empty streams");
    }
    Box::pin(futures::stream::select_all(streams))
}
