use crate::environment::Environment;
use crate::error::{
    unexpected_response, user_input, FetchMarketError, FetchMidpointError, FetchOrderBookError,
    ListEventsError, ListMarketsError, RateLimitError, TransportError, UserInputError,
};
use crate::http::ServiceClient;
use crate::pagination::{ListEventsPaginator, ListMarketsPaginator, Page, Paginator};
use crate::params::{as_reqwest_pairs, events_query, markets_query};
use polymarket_bindings::clob::{MidpointResponse, OrderBook};
use polymarket_bindings::gamma::{
    Event, GammaMarket, ListEventsKeysetRaw, ListEventsKeysetResponse, ListMarketsKeysetRaw,
    ListMarketsKeysetResponse, Market,
};
use polymarket_types::{MarketId, PaginationCursor, TokenId};

/// Read-only Polymarket client for discovery and market data.
#[derive(Clone)]
pub struct PublicClient {
    environment: Environment,
    gamma: ServiceClient,
    clob: ServiceClient,
    #[cfg(feature = "account")]
    pub(crate) data: polymarket_client_sdk_v2::data::Client,
    #[cfg(feature = "websockets")]
    pub(crate) ws: std::sync::Arc<crate::subscriptions::WebSocketClients>,
}

/// Builder for [`PublicClient`].
#[derive(Clone, Debug, Default)]
pub struct PublicClientBuilder {
    environment: Option<Environment>,
}

impl PublicClientBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn environment(mut self, environment: Environment) -> Self {
        self.environment = Some(environment);
        self
    }

    pub fn build(self) -> Result<PublicClient, TransportError> {
        PublicClient::with_environment(self.environment.unwrap_or_else(Environment::production))
    }
}

impl PublicClient {
    pub fn new(environment: Environment) -> Self {
        Self::with_environment(environment).expect("failed to construct HTTP clients")
    }

    pub fn builder() -> PublicClientBuilder {
        PublicClientBuilder::new()
    }

    pub fn with_environment(environment: Environment) -> Result<Self, TransportError> {
        Ok(Self {
            gamma: ServiceClient::new(environment.gamma)?,
            clob: ServiceClient::new(environment.clob)?,
            #[cfg(feature = "account")]
            data: polymarket_client_sdk_v2::data::Client::new(environment.data)
                .map_err(|e| TransportError(e.to_string()))?,
            #[cfg(feature = "websockets")]
            ws: std::sync::Arc::new(
                crate::subscriptions::WebSocketClients::new(&environment)
                    .map_err(|e| TransportError(e.to_string()))?,
            ),
            environment,
        })
    }

    #[must_use]
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn list_markets(
        &self,
        request: ListMarketsRequest,
    ) -> Result<ListMarketsPaginator, UserInputError> {
        validate_page_size(request.page_size)?;

        let gamma = self.gamma.clone();
        let base_request = request.clone();
        let initial_cursor = request.cursor;

        Ok(Paginator::new(
            move |cursor| {
                let gamma = gamma.clone();
                let mut req = base_request.clone();
                req.cursor = cursor;
                Box::pin(async move { fetch_markets_page(&gamma, &req).await })
            },
            initial_cursor,
        ))
    }

    pub async fn fetch_market(
        &self,
        request: FetchMarketRequest,
    ) -> Result<Market, FetchMarketError> {
        let path = match request {
            FetchMarketRequest::Id { id } => {
                let id = MarketId::parse(id)
                    .map_err(|e| FetchMarketError::from(user_input(e.message)))?;
                format!("/markets/{id}")
            }
            FetchMarketRequest::Slug { slug } => {
                if slug.trim().is_empty() {
                    return Err(FetchMarketError::from(user_input("slug cannot be empty")));
                }
                format!("/markets/slug/{slug}")
            }
            FetchMarketRequest::Url { url } => {
                let slug = parse_polymarket_slug(&url, "market")
                    .map_err(|e| FetchMarketError::from(user_input(e)))?;
                format!("/markets/slug/{slug}")
            }
        };

        let response = self
            .gamma
            .get(&path, &[])
            .await
            .map_err(FetchMarketError::from)?;
        let response = ServiceClient::ensure_success(response)
            .await
            .map_err(FetchMarketError::from)?;
        let raw: GammaMarket = ServiceClient::json(response)
            .await
            .map_err(FetchMarketError::from)?;

        polymarket_bindings::gamma::try_normalize_market(raw)
            .map_err(|e| FetchMarketError::from(unexpected_response(e)))
    }

    pub fn list_events(
        &self,
        request: ListEventsRequest,
    ) -> Result<ListEventsPaginator, UserInputError> {
        validate_page_size(request.page_size)?;

        let gamma = self.gamma.clone();
        let base_request = request.clone();
        let initial_cursor = request.cursor;

        Ok(Paginator::new(
            move |cursor| {
                let gamma = gamma.clone();
                let mut req = base_request.clone();
                req.cursor = cursor;
                Box::pin(async move { fetch_events_page(&gamma, &req).await })
            },
            initial_cursor,
        ))
    }

    pub async fn fetch_midpoint(
        &self,
        request: FetchMidpointRequest,
    ) -> Result<String, FetchMidpointError> {
        let token_id = TokenId::parse(request.token_id)
            .map_err(|e| FetchMidpointError::from(user_input(e.message)))?;

        let query = vec![("token_id", token_id.as_str().to_string())];
        let response = self
            .clob
            .get("/midpoint", &query)
            .await
            .map_err(FetchMidpointError::from)?;
        let response = ServiceClient::ensure_success(response)
            .await
            .map_err(FetchMidpointError::from)?;
        let parsed: MidpointResponse = ServiceClient::json(response)
            .await
            .map_err(FetchMidpointError::from)?;

        parsed
            .into_mid()
            .map_err(|e| FetchMidpointError::from(unexpected_response(e)))
    }

    pub async fn fetch_order_book(
        &self,
        request: FetchOrderBookRequest,
    ) -> Result<OrderBook, FetchOrderBookError> {
        let token_id = TokenId::parse(request.token_id)
            .map_err(|e| FetchOrderBookError::from(user_input(e.message)))?;

        let query = vec![("token_id", token_id.as_str().to_string())];
        let response = self
            .clob
            .get("/book", &query)
            .await
            .map_err(FetchOrderBookError::from)?;
        let response = ServiceClient::ensure_success(response)
            .await
            .map_err(FetchOrderBookError::from)?;
        ServiceClient::json(response)
            .await
            .map_err(FetchOrderBookError::from)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ListMarketsRequest {
    pub closed: Option<bool>,
    pub page_size: Option<u32>,
    pub cursor: Option<PaginationCursor>,
    pub order: Option<String>,
    pub ascending: Option<bool>,
    pub slug: Option<Vec<String>>,
    pub tag_id: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct ListEventsRequest {
    pub closed: Option<bool>,
    pub page_size: Option<u32>,
    pub cursor: Option<PaginationCursor>,
    pub order: Option<String>,
    pub ascending: Option<bool>,
    pub featured: Option<bool>,
}

pub enum FetchMarketRequest {
    Id { id: String },
    Slug { slug: String },
    Url { url: String },
}

pub struct FetchMidpointRequest {
    pub token_id: String,
}

pub struct FetchOrderBookRequest {
    pub token_id: String,
}

async fn fetch_markets_page(
    gamma: &ServiceClient,
    request: &ListMarketsRequest,
) -> Result<Page<Vec<Market>>, ListMarketsError> {
    let query = markets_query(request);
    let pairs = as_reqwest_pairs(&query);
    let response = gamma.get("/markets/keyset", &pairs).await?;
    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(ListMarketsError::from(RateLimitError));
    }
    let response = ServiceClient::ensure_success(response).await?;
    let raw: ListMarketsKeysetRaw = ServiceClient::json(response).await?;
    let parsed = ListMarketsKeysetResponse::from_raw(raw);

    Ok(Page {
        has_more: parsed.next_cursor.is_some(),
        next_cursor: parsed.next_cursor,
        items: parsed.items,
    })
}

async fn fetch_events_page(
    gamma: &ServiceClient,
    request: &ListEventsRequest,
) -> Result<Page<Vec<Event>>, ListEventsError> {
    let query = events_query(request);
    let pairs = as_reqwest_pairs(&query);
    let response = gamma.get("/events/keyset", &pairs).await?;
    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(ListEventsError::from(RateLimitError));
    }
    let response = ServiceClient::ensure_success(response).await?;
    let raw: ListEventsKeysetRaw = ServiceClient::json(response).await?;
    let parsed = ListEventsKeysetResponse::from_raw(raw);

    Ok(Page {
        has_more: parsed.next_cursor.is_some(),
        next_cursor: parsed.next_cursor,
        items: parsed.items,
    })
}

fn validate_page_size(page_size: Option<u32>) -> Result<(), UserInputError> {
    if let Some(size) = page_size {
        if size == 0 {
            return Err(user_input("page_size must be positive"));
        }
    }
    Ok(())
}

fn parse_polymarket_slug(url: &str, kind: &str) -> Result<String, String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("invalid url: {e}"))?;
    let segments: Vec<_> = parsed
        .path_segments()
        .ok_or_else(|| "url has no path".to_string())?
        .filter(|s| !s.is_empty())
        .collect();

    let pos = segments
        .iter()
        .position(|s| *s == kind)
        .ok_or_else(|| format!("url does not contain /{kind}/ segment"))?;
    let slug = segments
        .get(pos + 1)
        .ok_or_else(|| format!("missing slug after /{kind}/"))?;
    Ok((*slug).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_market_url() {
        let slug = parse_polymarket_slug(
            "https://polymarket.com/market/eth-flipped-in-2026",
            "market",
        )
        .unwrap();
        assert_eq!(slug, "eth-flipped-in-2026");
    }
}
