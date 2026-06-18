//! Unified Polymarket Rust SDK.
//!
//! Modeled after the official TypeScript SDK ([`@polymarket/client`](https://github.com/Polymarket/ts-sdk)).
//!
//! # Clients
//!
//! - [`PublicClient`] — read-only discovery, market data, and (with `account`) portfolio reads
//! - [`SecureClient`] — authenticated trading, notifications, rewards, CTF wallet ops, and user websockets
//!
//! # Feature flags
//!
//! | Feature | Enables |
//! |---------|---------|
//! | *(default)* | HTTP discovery + CLOB market data |
//! | `account` | Data API reads on [`PublicClient`] |
//! | `websockets` | Realtime `subscribe()` on [`PublicClient`] |
//! | `secure` | `account` + `websockets` + [`SecureClient`] trading and wallet ops |
//!
//! # Quickstart
//!
//! ```no_run
//! use polymarket_client::{Environment, PublicClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = PublicClient::new(Environment::production());
//!     let mut paginator = client.list_markets(polymarket_client::ListMarketsRequest {
//!         closed: Some(false),
//!         page_size: Some(5),
//!         ..Default::default()
//!     })?;
//!     let page = paginator.first_page().await?;
//!     for market in &page.items {
//!         println!("{}: {}", market.id, market.question.as_deref().unwrap_or(""));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Realtime (websockets)
//!
//! ```no_run
//! # #[cfg(feature = "websockets")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use futures::StreamExt as _;
//! use polymarket_client::{Environment, MarketSubscription, PublicClient, SubscriptionSpec};
//!
//! let client = PublicClient::new(Environment::production());
//! let mut handle = client.subscribe(vec![SubscriptionSpec::Market(MarketSubscription {
//!     token_ids: vec!["123".into()],
//!     custom_feature_enabled: false,
//! })])?;
//! if let Some(Ok(event)) = handle.next().await {
//!     println!("{event:?}");
//! }
//! handle.close();
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)]

mod environment;
mod error;
mod http;
mod pagination;
mod params;
mod public_client;

#[cfg(feature = "account")]
pub(crate) mod account;
#[cfg(feature = "account")]
pub(crate) mod account_client;

#[cfg(feature = "secure")]
mod secure;

#[cfg(feature = "websockets")]
mod subscriptions;

pub use environment::Environment;
pub use error::{
    unexpected_response, user_input, Error, FetchMarketError, FetchMidpointError,
    FetchOrderBookError, ListEventsError, ListMarketsError, RateLimitError, RequestRejectedError,
    TransportError, UnexpectedResponseError, UserInputError,
};
pub use pagination::{Page, Paginator};
pub use polymarket_bindings::clob::{OrderBook, OrderBookLevel};
pub use polymarket_bindings::gamma::{Event, Market};
pub use polymarket_bindings::{OrderSide, OrderType};
pub use polymarket_types::{
    CtfConditionId, DecimalString, EventId, EvmAddress, MarketId, PaginationCursor, TokenId,
};
pub use public_client::{
    FetchMarketRequest, FetchMidpointRequest, FetchOrderBookRequest, ListEventsRequest,
    ListMarketsRequest, PublicClient, PublicClientBuilder,
};

#[cfg(feature = "account")]
pub use account::{
    Activity, FetchPortfolioValueError, FetchPortfolioValueRequest, ListActivityError,
    ListActivityRequest, ListPositionsError, ListPositionsRequest, PortfolioValue, Position,
};

#[cfg(feature = "account")]
pub use account_client::{ListActivityPaginator, ListPositionsPaginator};

#[cfg(feature = "secure")]
pub use polymarket_client_sdk_v2::PRIVATE_KEY_VAR;

#[cfg(feature = "secure")]
pub use secure::{
    AccountTrade, ApiCredentials, BuildSecureClientError, CancelMarketOrdersRequest,
    CancelOrderError, CancelOrderRequest, CancelOrderResponse, CurrentReward,
    FetchNotificationsError, FetchOrderError, FetchOrderRequest, FetchOrderScoringError,
    FetchOrderScoringRequest, ListAccountTradesError, ListAccountTradesRequest,
    ListCurrentRewardsError, ListOpenOrdersError, ListOpenOrdersRequest, MarketOrderType,
    MergePositionsRequest, Notification, OpenOrder, PlaceLimitOrderRequest,
    PlaceMarketOrderRequest, PlaceOrderError, PlaceOrderResponse, RedeemPositionsRequest,
    SecureClient, SecureClientBuilder, SetupTradingApprovalsError, SplitPositionRequest,
    TransactionOutcome, WalletOperationError,
};

#[cfg(feature = "websockets")]
pub use subscriptions::{
    CommentsSubscription, CryptoPricesSubscription, EquityPricesSubscription, MarketStreamEvent,
    MarketSubscription, SportsStreamEvent, StreamEvent, SubscribeError, SubscriptionHandle,
    SubscriptionSpec, UserStreamEvent, UserSubscription,
};
