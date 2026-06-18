//! Authenticated client, credentials, and trading (requires `secure` feature).

mod credentials;
mod secure_client;

#[cfg(feature = "secure")]
mod account;
#[cfg(feature = "secure")]
mod wallet;

#[cfg(all(feature = "secure", feature = "websockets"))]
mod subscribe;

pub use credentials::ApiCredentials;
pub use secure_client::{
    BuildSecureClientError, CancelMarketOrdersRequest, CancelOrderError, CancelOrderRequest,
    CancelOrderResponse, FetchOrderError, FetchOrderRequest, ListOpenOrdersError,
    ListOpenOrdersRequest, MarketOrderType, OpenOrder, PlaceLimitOrderRequest,
    PlaceMarketOrderRequest, PlaceOrderError, PlaceOrderResponse, SecureClient,
    SecureClientBuilder, SetupTradingApprovalsError,
};

#[cfg(feature = "secure")]
pub use account::{
    AccountTrade, CurrentReward, FetchNotificationsError, FetchOrderScoringError,
    FetchOrderScoringRequest, ListAccountTradesError, ListAccountTradesRequest,
    ListCurrentRewardsError, Notification,
};

#[cfg(feature = "secure")]
pub use wallet::{
    MergePositionsRequest, RedeemPositionsRequest, SplitPositionRequest, TransactionOutcome,
    WalletOperationError,
};
