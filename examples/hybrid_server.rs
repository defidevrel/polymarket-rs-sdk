//! Hybrid adapter server — any front-end (e.g. Solana) → Polymarket settlement on Polygon.
//!
//! Public routes work without credentials. Trading routes require `POLYMARKET_PRIVATE_KEY`
//! and `POLYMARKET_PLACE_ORDER=1`.
//!
//! ```bash
//! cargo run --example hybrid_server --features secure
//!
//! # With trading enabled:
//! POLYMARKET_PRIVATE_KEY=0x… POLYMARKET_PLACE_ORDER=1 \
//!   cargo run --example hybrid_server --features secure
//! ```
//!
//! Example requests:
//! ```text
//! curl http://127.0.0.1:8080/health
//! curl 'http://127.0.0.1:8080/v1/markets?limit=3'
//! curl http://127.0.0.1:8080/v1/book/<token_id>
//! curl -X POST http://127.0.0.1:8080/v1/orders \
//!   -H 'Content-Type: application/json' \
//!   -H 'X-Solana-Address: <base58-pubkey>' \
//!   -d '{"token_id":"…","side":"buy","price":0.01,"size":5.0,"post_only":true}'
//! ```

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use polymarket_client::{
    Environment, FetchOrderBookRequest, ListMarketsRequest, OrderSide, PlaceLimitOrderRequest,
    PublicClient, SecureClient, PRIVATE_KEY_VAR,
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone)]
struct AppState {
    public: PublicClient,
    secure: Option<Arc<SecureClient>>,
    trading_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct MarketsQuery {
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    5
}

#[derive(Debug, Deserialize)]
struct PlaceOrderBody {
    token_id: String,
    side: String,
    price: f64,
    size: f64,
    #[serde(default)]
    post_only: bool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    settlement_chain: &'static str,
    polygon_wallet: Option<String>,
    trading_enabled: bool,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Serialize)]
struct PlaceOrderResponseBody {
    solana_address: Option<String>,
    polygon_wallet: String,
    ok: bool,
    order_id: Option<String>,
    message: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let public = PublicClient::new(Environment::production());
    let trading_enabled = std::env::var("POLYMARKET_PLACE_ORDER").as_deref() == Ok("1");

    let secure = if let Ok(private_key) = std::env::var(PRIVATE_KEY_VAR) {
        let secure = SecureClient::builder()
            .environment(Environment::production())
            .private_key(private_key)
            .build()
            .await?;
        secure.setup_trading_approvals().await?;
        info!(wallet = %secure.wallet(), "Polygon trading wallet ready");
        Some(Arc::new(secure))
    } else {
        info!("No {PRIVATE_KEY_VAR} — read-only mode (markets + order book only)");
        None
    };

    let state = AppState {
        public,
        secure,
        trading_enabled,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/markets", get(list_markets))
        .route("/v1/book/{token_id}", get(fetch_book))
        .route("/v1/orders", post(place_order))
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!(%addr, "hybrid adapter listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        settlement_chain: "polygon",
        polygon_wallet: state
            .secure
            .as_ref()
            .map(|client| client.wallet().to_string()),
        trading_enabled: state.trading_enabled && state.secure.is_some(),
    })
}

async fn list_markets(
    State(state): State<AppState>,
    Query(query): Query<MarketsQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit = query.limit.clamp(1, 50);
    let mut paginator = state
        .public
        .list_markets(ListMarketsRequest {
            closed: Some(false),
            page_size: Some(limit),
            ..Default::default()
        })
        .map_err(|error| ApiError::bad_request(error.to_string()))?;

    let page = paginator
        .first_page()
        .await
        .map_err(|error| ApiError::upstream(error.to_string()))?;
    Ok(Json(serde_json::json!({
        "items": page.items,
        "has_more": page.has_more,
    })))
}

async fn fetch_book(
    State(state): State<AppState>,
    Path(token_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let book = state
        .public
        .fetch_order_book(FetchOrderBookRequest { token_id })
        .await
        .map_err(|error| ApiError::upstream(error.to_string()))?;
    Ok(Json(serde_json::json!({
        "market": book.market.as_str(),
        "token_id": book.token_id.as_str(),
        "timestamp": book.timestamp,
        "bids": book.bids.iter().map(level_json).collect::<Vec<_>>(),
        "asks": book.asks.iter().map(level_json).collect::<Vec<_>>(),
        "min_order_size": book.min_order_size.as_str(),
        "tick_size": book.tick_size.as_str(),
        "neg_risk": book.neg_risk,
        "last_trade_price": book
            .last_trade_price
            .as_ref()
            .map(polymarket_client::DecimalString::as_str),
        "hash": book.hash,
    })))
}

fn level_json(level: &polymarket_client::OrderBookLevel) -> serde_json::Value {
    serde_json::json!({
        "price": level.price,
        "size": level.size,
    })
}

async fn place_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PlaceOrderBody>,
) -> Result<Json<PlaceOrderResponseBody>, ApiError> {
    let solana_address = headers
        .get("x-solana-address")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    if solana_address.is_some() {
        info!(solana_address = ?solana_address, "order request from hybrid front-end");
    }

    if !state.trading_enabled {
        return Err(ApiError::forbidden(
            "set POLYMARKET_PLACE_ORDER=1 to enable live order placement",
        ));
    }

    let secure = state.secure.as_ref().ok_or_else(|| {
        ApiError::service_unavailable(format!("set {PRIVATE_KEY_VAR} for trading"))
    })?;

    let side = match body.side.to_ascii_lowercase().as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        other => {
            return Err(ApiError::bad_request(format!(
                "side must be buy or sell, got {other}"
            )));
        }
    };

    let response = secure
        .place_limit_order(PlaceLimitOrderRequest {
            token_id: body.token_id,
            side,
            price: body.price,
            size: body.size,
            expiration: None,
            post_only: body.post_only,
        })
        .await
        .map_err(|error| ApiError::upstream(error.to_string()))?;

    Ok(Json(PlaceOrderResponseBody {
        solana_address,
        polygon_wallet: secure.wallet().to_string(),
        ok: response.ok,
        order_id: response.order_id,
        message: response.message,
    }))
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: message.into(),
        }
    }

    fn upstream(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}
