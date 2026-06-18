#[cfg(feature = "account")]
use std::str::FromStr as _;

#[cfg(feature = "account")]
use polymarket_client_sdk_v2::data::types::request::{
    ActivityRequest, PositionsRequest, ValueRequest,
};
#[cfg(feature = "account")]
use polymarket_client_sdk_v2::types::{Address, B256};

#[cfg(feature = "account")]
use crate::account::{
    decode_offset_cursor, mappers, next_offset_cursor, validate_page_size, validate_user, Activity,
    FetchPortfolioValueError, FetchPortfolioValueRequest, ListActivityError, ListActivityRequest,
    ListPositionsError, ListPositionsRequest, PortfolioValue, Position,
};
#[cfg(feature = "account")]
use crate::error::user_input;
#[cfg(feature = "account")]
use crate::pagination::{Page, Paginator};
#[cfg(feature = "account")]
use crate::public_client::PublicClient;

#[cfg(feature = "account")]
pub type ListPositionsPaginator = Paginator<Vec<Position>, ListPositionsError>;

#[cfg(feature = "account")]
pub type ListActivityPaginator = Paginator<Vec<Activity>, ListActivityError>;

#[cfg(feature = "account")]
impl PublicClient {
    pub fn list_positions(
        &self,
        request: ListPositionsRequest,
    ) -> Result<ListPositionsPaginator, ListPositionsError> {
        validate_user(&request.user)?;

        let page_size = validate_page_size(request.page_size).map_err(ListPositionsError::from)?;
        let data = self.data.clone();
        let user = parse_address(&request.user).map_err(ListPositionsError::from)?;
        let initial_cursor = request.cursor;

        Ok(Paginator::new(
            move |cursor| {
                let data = data.clone();
                Box::pin(async move {
                    let state = decode_offset_cursor(cursor.as_ref(), page_size)
                        .map_err(ListPositionsError::from)?;
                    let limit = i32::try_from(state.page_size.saturating_add(1))
                        .map_err(|_| ListPositionsError::from(user_input("page_size too large")))?;
                    let offset = i32::try_from(state.offset)
                        .map_err(|_| ListPositionsError::from(user_input("offset too large")))?;

                    let req = PositionsRequest::builder()
                        .user(user)
                        .limit(limit)
                        .map_err(|e| ListPositionsError::Data(e.to_string()))?
                        .offset(offset)
                        .map_err(|e| ListPositionsError::Data(e.to_string()))?
                        .build();

                    let items = data
                        .positions(&req)
                        .await
                        .map_err(|e| ListPositionsError::Data(e.to_string()))?;

                    let page_size_usize = state.page_size as usize;
                    let has_more = items.len() > page_size_usize;
                    let page_items = items
                        .into_iter()
                        .take(page_size_usize)
                        .map(mappers::map_position)
                        .collect();

                    Ok(Page {
                        items: page_items,
                        has_more,
                        next_cursor: has_more.then(|| next_offset_cursor(state)),
                    })
                })
            },
            initial_cursor,
        ))
    }

    pub async fn fetch_portfolio_value(
        &self,
        request: FetchPortfolioValueRequest,
    ) -> Result<Vec<PortfolioValue>, FetchPortfolioValueError> {
        validate_user(&request.user)?;
        let user = parse_address(&request.user).map_err(FetchPortfolioValueError::from)?;

        let req = if request.markets.is_empty() {
            ValueRequest::builder().user(user).build()
        } else {
            let parsed: Result<Vec<B256>, _> =
                request.markets.iter().map(|m| parse_b256(m)).collect();
            ValueRequest::builder()
                .user(user)
                .markets(parsed.map_err(FetchPortfolioValueError::from)?)
                .build()
        };

        let values = self
            .data
            .value(&req)
            .await
            .map_err(|e| FetchPortfolioValueError::Data(e.to_string()))?;

        Ok(values
            .into_iter()
            .map(mappers::map_portfolio_value)
            .collect())
    }

    pub fn list_activity(
        &self,
        request: ListActivityRequest,
    ) -> Result<ListActivityPaginator, ListActivityError> {
        validate_user(&request.user)?;

        let page_size = validate_page_size(request.page_size).map_err(ListActivityError::from)?;
        let data = self.data.clone();
        let user = parse_address(&request.user).map_err(ListActivityError::from)?;
        let initial_cursor = request.cursor;

        Ok(Paginator::new(
            move |cursor| {
                let data = data.clone();
                Box::pin(async move {
                    let state = decode_offset_cursor(cursor.as_ref(), page_size)
                        .map_err(ListActivityError::from)?;
                    let limit = i32::try_from(state.page_size.saturating_add(1))
                        .map_err(|_| ListActivityError::from(user_input("page_size too large")))?;
                    let offset = i32::try_from(state.offset)
                        .map_err(|_| ListActivityError::from(user_input("offset too large")))?;

                    let req = ActivityRequest::builder()
                        .user(user)
                        .limit(limit)
                        .map_err(|e| ListActivityError::Data(e.to_string()))?
                        .offset(offset)
                        .map_err(|e| ListActivityError::Data(e.to_string()))?
                        .build();

                    let items = data
                        .activity(&req)
                        .await
                        .map_err(|e| ListActivityError::Data(e.to_string()))?;

                    let page_size_usize = state.page_size as usize;
                    let has_more = items.len() > page_size_usize;
                    let page_items = items
                        .into_iter()
                        .take(page_size_usize)
                        .map(mappers::map_activity)
                        .collect();

                    Ok(Page {
                        items: page_items,
                        has_more,
                        next_cursor: has_more.then(|| next_offset_cursor(state)),
                    })
                })
            },
            initial_cursor,
        ))
    }
}

#[cfg(feature = "account")]
fn parse_address(value: &str) -> Result<Address, crate::error::UserInputError> {
    Address::from_str(value).map_err(|e| user_input(format!("invalid address: {e}")))
}

#[cfg(feature = "account")]
fn parse_b256(value: &str) -> Result<B256, crate::error::UserInputError> {
    B256::from_str(value).map_err(|e| user_input(format!("invalid market id: {e}")))
}
