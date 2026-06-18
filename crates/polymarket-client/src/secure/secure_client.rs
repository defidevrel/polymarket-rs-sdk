use std::ops::Deref;
use std::str::FromStr as _;

use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer as _;
use chrono::{DateTime, Utc};
use polymarket_bindings::OrderSide;
use polymarket_client_sdk_v2::auth::state::Authenticated;
use polymarket_client_sdk_v2::auth::Normal;
use polymarket_client_sdk_v2::clob::types::request::{
    CancelMarketOrderRequest, OrdersRequest, UpdateBalanceAllowanceRequest,
};
use polymarket_client_sdk_v2::clob::types::response::{OpenOrderResponse, PostOrderResponse};
use polymarket_client_sdk_v2::clob::types::{OrderType, Side, SignatureType};
use polymarket_client_sdk_v2::clob::{Client as ClobClient, Config};
use polymarket_client_sdk_v2::types::{Address, Decimal, B256, U256};
use polymarket_client_sdk_v2::POLYGON;
use rust_decimal::prelude::FromPrimitive as _;

use crate::environment::Environment;
use crate::error::{user_input, UserInputError};
use crate::public_client::PublicClient;
use crate::secure::credentials::ApiCredentials;

type AuthenticatedClob = ClobClient<Authenticated<Normal>>;

/// Errors while constructing a [`SecureClient`].
#[derive(Debug, thiserror::Error)]
pub enum BuildSecureClientError {
    #[error("private key is required")]
    MissingPrivateKey,
    #[error("invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("invalid credentials: {0}")]
    InvalidCredentials(String),
    #[error("HTTP client error: {0}")]
    Http(String),
    #[error("SDK error: {0}")]
    Sdk(String),
}

macro_rules! secure_action_error {
    ($name:ident) => {
        #[derive(Debug, thiserror::Error, Clone)]
        pub enum $name {
            #[error(transparent)]
            UserInput(#[from] UserInputError),
            #[error("signing or SDK error: {0}")]
            Sdk(String),
        }

        impl $name {
            #[must_use]
            pub fn is_error(err: &(dyn std::error::Error + 'static)) -> bool {
                err.downcast_ref::<Self>().is_some()
                    || err.downcast_ref::<UserInputError>().is_some()
            }
        }
    };
}

secure_action_error!(PlaceOrderError);
secure_action_error!(FetchOrderError);
secure_action_error!(ListOpenOrdersError);
secure_action_error!(CancelOrderError);
secure_action_error!(SetupTradingApprovalsError);

pub type PlaceOrderResponse = OrderPlacementResponse;

#[derive(Clone, Debug)]
pub struct OrderPlacementResponse {
    pub ok: bool,
    pub order_id: Option<String>,
    pub code: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PlaceLimitOrderRequest {
    pub token_id: String,
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    /// Unix timestamp (seconds). Requires GTD order type.
    pub expiration: Option<i64>,
    pub post_only: bool,
}

#[derive(Clone, Debug)]
pub struct PlaceMarketOrderRequest {
    pub token_id: String,
    pub side: OrderSide,
    /// USDC amount for buy-side market orders.
    pub amount: Option<f64>,
    /// Share amount for sell-side market orders.
    pub shares: Option<f64>,
    pub order_type: MarketOrderType,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum MarketOrderType {
    #[default]
    Fak,
    Fok,
}

#[derive(Clone, Debug, Default)]
pub struct FetchOrderRequest {
    pub order_id: String,
}

#[derive(Clone, Debug, Default)]
pub struct ListOpenOrdersRequest {
    pub market: Option<String>,
    pub token_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OpenOrder {
    pub order_id: String,
    pub token_id: String,
    pub side: OrderSide,
    pub price: String,
    pub original_size: String,
    pub size_matched: String,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct CancelOrderRequest {
    pub order_id: String,
}

#[derive(Clone, Debug, Default)]
pub struct CancelMarketOrdersRequest {
    pub token_id: Option<String>,
    pub market: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CancelOrderResponse {
    pub canceled: Vec<String>,
}

/// Authenticated Polymarket client for trading and account-scoped CLOB operations.
pub struct SecureClient {
    public: PublicClient,
    pub(crate) clob: AuthenticatedClob,
    pub(crate) signer: PrivateKeySigner,
    credentials: ApiCredentials,
    wallet: Address,
}

/// Builder for [`SecureClient`].
#[derive(Clone, Debug, Default)]
pub struct SecureClientBuilder {
    environment: Option<Environment>,
    private_key: Option<String>,
    credentials: Option<ApiCredentials>,
    funder: Option<Address>,
    signature_type: Option<SignatureType>,
}

impl SecureClientBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn environment(mut self, environment: Environment) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Hex-encoded secp256k1 private key (`0x…`).
    #[must_use]
    pub fn private_key(mut self, private_key: impl Into<String>) -> Self {
        self.private_key = Some(private_key.into());
        self
    }

    /// Reuse stored L2 credentials from a prior session.
    #[must_use]
    pub fn credentials(mut self, credentials: ApiCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Polymarket account wallet (funder). Required for proxy/safe/deposit wallets.
    #[must_use]
    pub fn wallet(mut self, wallet: Address) -> Self {
        self.funder = Some(wallet);
        self
    }

    #[must_use]
    pub fn signature_type(mut self, signature_type: SignatureType) -> Self {
        self.signature_type = Some(signature_type);
        self
    }

    pub async fn build(self) -> Result<SecureClient, BuildSecureClientError> {
        let environment = self.environment.unwrap_or_else(Environment::production);
        let private_key = self
            .private_key
            .ok_or(BuildSecureClientError::MissingPrivateKey)?;
        let signer = PrivateKeySigner::from_str(&private_key)
            .map_err(|e| BuildSecureClientError::InvalidPrivateKey(e.to_string()))?
            .with_chain_id(Some(POLYGON));

        let public = PublicClient::with_environment(environment.clone())
            .map_err(|e| BuildSecureClientError::Http(e.0))?;

        let config = Config::builder().use_server_time(true).build();
        let unauth = ClobClient::new(environment.clob, config)
            .map_err(|e| BuildSecureClientError::Sdk(e.to_string()))?;

        let credentials = if let Some(credentials) = self.credentials.clone() {
            credentials
        } else {
            let sdk_creds = unauth
                .create_or_derive_api_key(&signer, None)
                .await
                .map_err(|e| BuildSecureClientError::Sdk(e.to_string()))?;
            ApiCredentials::from_sdk(&sdk_creds)
        };

        let sdk_creds = credentials
            .to_sdk_credentials()
            .map_err(|e| BuildSecureClientError::InvalidCredentials(e.to_string()))?;

        let mut auth_builder = unauth
            .authentication_builder(&signer)
            .credentials(sdk_creds);

        if let Some(funder) = self.funder {
            auth_builder = auth_builder.funder(funder);
        }
        if let Some(signature_type) = self.signature_type {
            auth_builder = auth_builder.signature_type(signature_type);
        }

        let clob = auth_builder
            .authenticate()
            .await
            .map_err(|e| BuildSecureClientError::Sdk(e.to_string()))?;
        let wallet = self.funder.unwrap_or_else(|| signer.address());

        Ok(SecureClient {
            public,
            clob,
            signer,
            credentials,
            wallet,
        })
    }
}

impl SecureClient {
    #[must_use]
    pub fn builder() -> SecureClientBuilder {
        SecureClientBuilder::new()
    }

    #[must_use]
    pub fn public(&self) -> &PublicClient {
        &self.public
    }

    #[must_use]
    pub fn credentials(&self) -> &ApiCredentials {
        &self.credentials
    }

    #[must_use]
    pub fn wallet(&self) -> Address {
        self.wallet
    }

    #[must_use]
    pub fn environment(&self) -> &Environment {
        self.public.environment()
    }

    /// Syncs balance/allowance state with the CLOB. Call before first trade.
    pub async fn setup_trading_approvals(&self) -> Result<(), SetupTradingApprovalsError> {
        self.clob
            .update_balance_allowance(UpdateBalanceAllowanceRequest::default())
            .await
            .map_err(|e| SetupTradingApprovalsError::Sdk(e.to_string()))?;
        Ok(())
    }

    pub async fn place_limit_order(
        &self,
        request: PlaceLimitOrderRequest,
    ) -> Result<PlaceOrderResponse, PlaceOrderError> {
        validate_positive(request.price, "price")?;
        validate_positive(request.size, "size")?;

        let token_id = parse_token_id(&request.token_id)?;
        let price = decimal_from_f64(request.price, "price")?;
        let size = decimal_from_f64(request.size, "size")?;

        let mut builder = self
            .clob
            .limit_order()
            .token_id(token_id)
            .price(price)
            .size(size)
            .side(map_side(request.side))
            .post_only(request.post_only);

        if let Some(expiration) = request.expiration {
            let expiry = DateTime::<Utc>::from_timestamp(expiration, 0).ok_or_else(|| {
                PlaceOrderError::UserInput(user_input("expiration must be a valid unix timestamp"))
            })?;
            builder = builder.order_type(OrderType::GTD).expiration(expiry);
        }

        let order = builder
            .build()
            .await
            .map_err(|e| PlaceOrderError::Sdk(e.to_string()))?;
        let signed = self
            .clob
            .sign(&self.signer, order)
            .await
            .map_err(|e| PlaceOrderError::Sdk(e.to_string()))?;
        let response = self
            .clob
            .post_order(signed)
            .await
            .map_err(|e| PlaceOrderError::Sdk(e.to_string()))?;
        Ok(map_post_order_response(response))
    }

    pub async fn place_market_order(
        &self,
        request: PlaceMarketOrderRequest,
    ) -> Result<PlaceOrderResponse, PlaceOrderError> {
        let token_id = parse_token_id(&request.token_id)?;
        let side = map_side(request.side);

        let mut builder = self.clob.market_order().token_id(token_id).side(side);
        builder = match (request.amount, request.shares) {
            (Some(amount), None) => {
                validate_positive(amount, "amount")?;
                builder.amount(
                    polymarket_client_sdk_v2::clob::types::Amount::usdc(decimal_from_f64(
                        amount, "amount",
                    )?)
                    .map_err(|e| PlaceOrderError::Sdk(e.to_string()))?,
                )
            }
            (None, Some(shares)) => {
                validate_positive(shares, "shares")?;
                builder.amount(
                    polymarket_client_sdk_v2::clob::types::Amount::shares(decimal_from_f64(
                        shares, "shares",
                    )?)
                    .map_err(|e| PlaceOrderError::Sdk(e.to_string()))?,
                )
            }
            _ => {
                return Err(PlaceOrderError::UserInput(user_input(
                    "provide either amount (buy) or shares (sell) for market orders",
                )));
            }
        };

        builder = builder.order_type(match request.order_type {
            MarketOrderType::Fak => OrderType::FAK,
            MarketOrderType::Fok => OrderType::FOK,
        });

        let order = builder
            .build()
            .await
            .map_err(|e| PlaceOrderError::Sdk(e.to_string()))?;
        let signed = self
            .clob
            .sign(&self.signer, order)
            .await
            .map_err(|e| PlaceOrderError::Sdk(e.to_string()))?;
        let response = self
            .clob
            .post_order(signed)
            .await
            .map_err(|e| PlaceOrderError::Sdk(e.to_string()))?;
        Ok(map_post_order_response(response))
    }

    pub async fn fetch_order(
        &self,
        request: FetchOrderRequest,
    ) -> Result<OpenOrder, FetchOrderError> {
        if request.order_id.trim().is_empty() {
            return Err(FetchOrderError::UserInput(user_input(
                "order_id cannot be empty",
            )));
        }
        let order = self
            .clob
            .order(&request.order_id)
            .await
            .map_err(|e| FetchOrderError::Sdk(e.to_string()))?;
        Ok(map_open_order(order))
    }

    pub async fn list_open_orders(
        &self,
        request: ListOpenOrdersRequest,
    ) -> Result<Vec<OpenOrder>, ListOpenOrdersError> {
        let mut req = OrdersRequest::default();
        if let Some(market) = request.market {
            req.market = Some(parse_market_id(&market).map_err(ListOpenOrdersError::from)?);
        }
        if let Some(token_id) = request.token_id {
            req.asset_id = Some(parse_token_id(&token_id).map_err(ListOpenOrdersError::from)?);
        }

        let mut cursor = None;
        let mut all = Vec::new();
        loop {
            let page = self
                .clob
                .orders(&req, cursor.clone())
                .await
                .map_err(|e| ListOpenOrdersError::Sdk(e.to_string()))?;
            all.extend(page.data.into_iter().map(map_open_order));
            if page.next_cursor.is_empty() || page.next_cursor == "LTE=" {
                break;
            }
            cursor = Some(page.next_cursor);
        }
        Ok(all)
    }

    pub async fn cancel_order(
        &self,
        request: CancelOrderRequest,
    ) -> Result<CancelOrderResponse, CancelOrderError> {
        let response = self
            .clob
            .cancel_order(&request.order_id)
            .await
            .map_err(|e| CancelOrderError::Sdk(e.to_string()))?;
        Ok(CancelOrderResponse {
            canceled: response.canceled,
        })
    }

    pub async fn cancel_market_orders(
        &self,
        request: CancelMarketOrdersRequest,
    ) -> Result<CancelOrderResponse, CancelOrderError> {
        let mut req = CancelMarketOrderRequest::default();
        if let Some(token_id) = request.token_id {
            req.asset_id = Some(parse_token_id(&token_id).map_err(CancelOrderError::from)?);
        }
        if let Some(market) = request.market {
            req.market = Some(parse_market_id(&market).map_err(CancelOrderError::from)?);
        }
        let response = self
            .clob
            .cancel_market_orders(&req)
            .await
            .map_err(|e| CancelOrderError::Sdk(e.to_string()))?;
        Ok(CancelOrderResponse {
            canceled: response.canceled,
        })
    }
}

impl Deref for SecureClient {
    type Target = PublicClient;

    fn deref(&self) -> &Self::Target {
        &self.public
    }
}

fn map_side(side: OrderSide) -> Side {
    match side {
        OrderSide::Buy => Side::Buy,
        OrderSide::Sell => Side::Sell,
    }
}

fn parse_token_id(token_id: &str) -> Result<U256, UserInputError> {
    U256::from_str(token_id).map_err(|e| user_input(format!("invalid token_id: {e}")))
}

fn parse_market_id(market: &str) -> Result<B256, UserInputError> {
    B256::from_str(market).map_err(|e| user_input(format!("invalid market id: {e}")))
}

fn decimal_from_f64(value: f64, field: &str) -> Result<Decimal, UserInputError> {
    Decimal::from_f64(value).ok_or_else(|| user_input(format!("invalid {field}: {value}")))
}

fn validate_positive(value: f64, field: &str) -> Result<(), UserInputError> {
    if value <= 0.0 || !value.is_finite() {
        return Err(user_input(format!("{field} must be a positive number")));
    }
    Ok(())
}

fn map_post_order_response(response: PostOrderResponse) -> PlaceOrderResponse {
    PlaceOrderResponse {
        ok: response.success,
        order_id: if response.order_id.is_empty() {
            None
        } else {
            Some(response.order_id)
        },
        code: if response.success {
            None
        } else {
            Some("ORDER_REJECTED".into())
        },
        message: response.error_msg,
    }
}

fn map_open_order(order: OpenOrderResponse) -> OpenOrder {
    OpenOrder {
        order_id: order.id,
        token_id: order.asset_id.to_string(),
        side: match order.side {
            Side::Sell => OrderSide::Sell,
            _ => OrderSide::Buy,
        },
        price: order.price.to_string(),
        original_size: order.original_size.to_string(),
        size_matched: order.size_matched.to_string(),
        status: format!("{:?}", order.status),
    }
}
