//! Account data types, requests, and pagination helpers.

#![allow(clippy::redundant_pub_crate)]

use polymarket_types::PaginationCursor;

use crate::error::{user_input, UserInputError};

macro_rules! account_error {
    ($name:ident) => {
        #[derive(Debug, thiserror::Error, Clone)]
        pub enum $name {
            #[error(transparent)]
            UserInput(#[from] UserInputError),
            #[error("data API error: {0}")]
            Data(String),
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

account_error!(ListPositionsError);
account_error!(ListActivityError);
account_error!(FetchPortfolioValueError);

#[derive(Clone, Debug)]
pub struct Position {
    pub token_id: String,
    pub condition_id: String,
    pub size: String,
    pub avg_price: String,
    pub current_value: String,
    pub cash_pnl: String,
    pub percent_pnl: String,
    pub title: String,
    pub outcome: String,
    pub redeemable: bool,
    pub mergeable: bool,
}

#[derive(Clone, Debug)]
pub struct Activity {
    pub timestamp: i64,
    pub activity_type: String,
    pub condition_id: Option<String>,
    pub size: String,
    pub usdc_size: String,
    pub transaction_hash: String,
    pub title: Option<String>,
    pub outcome: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PortfolioValue {
    pub user: String,
    pub value: String,
}

#[derive(Clone, Debug, Default)]
pub struct ListPositionsRequest {
    pub user: String,
    pub markets: Vec<String>,
    pub page_size: Option<u32>,
    pub cursor: Option<PaginationCursor>,
    pub redeemable: Option<bool>,
    pub mergeable: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct ListActivityRequest {
    pub user: String,
    pub page_size: Option<u32>,
    pub cursor: Option<PaginationCursor>,
}

#[derive(Clone, Debug, Default)]
pub struct FetchPortfolioValueRequest {
    pub user: String,
    pub markets: Vec<String>,
}

pub(crate) struct OffsetCursorState {
    pub(crate) offset: u32,
    pub(crate) page_size: u32,
}

pub(crate) fn decode_offset_cursor(
    cursor: Option<&PaginationCursor>,
    default_page_size: u32,
) -> Result<OffsetCursorState, UserInputError> {
    let Some(cursor) = cursor else {
        return Ok(OffsetCursorState {
            offset: 0,
            page_size: default_page_size,
        });
    };

    let parts: Vec<&str> = cursor.as_str().split(':').collect();
    if parts.len() != 4 || parts[0] != "offset" || parts[2] != "page_size" {
        return Err(user_input("invalid pagination cursor"));
    }
    let offset = parts[1]
        .parse()
        .map_err(|_| user_input("invalid pagination cursor offset"))?;
    let page_size = parts[3]
        .parse()
        .map_err(|_| user_input("invalid pagination cursor page_size"))?;
    Ok(OffsetCursorState { offset, page_size })
}

pub(crate) fn encode_offset_cursor(state: OffsetCursorState) -> PaginationCursor {
    PaginationCursor::parse(format!(
        "offset:{}:page_size:{}",
        state.offset, state.page_size
    ))
    .expect("offset cursor is valid")
}

pub(crate) fn next_offset_cursor(state: OffsetCursorState) -> PaginationCursor {
    encode_offset_cursor(OffsetCursorState {
        offset: state.offset + state.page_size,
        page_size: state.page_size,
    })
}

pub(crate) fn validate_page_size(page_size: Option<u32>) -> Result<u32, UserInputError> {
    let size = page_size.unwrap_or(20);
    if !(1..=500).contains(&size) {
        return Err(user_input("page_size must be between 1 and 500"));
    }
    Ok(size)
}

pub(crate) fn validate_user(user: &str) -> Result<(), UserInputError> {
    if user.trim().is_empty() {
        return Err(user_input("user address cannot be empty"));
    }
    Ok(())
}

#[cfg(feature = "account")]
pub(crate) mod mappers {
    use polymarket_client_sdk_v2::data::types::response::{
        Activity as SdkActivity, Position as SdkPosition, Value as SdkValue,
    };

    use super::{Activity, PortfolioValue, Position};

    pub fn map_position(position: SdkPosition) -> Position {
        Position {
            token_id: position.asset.to_string(),
            condition_id: position.condition_id.to_string(),
            size: position.size.to_string(),
            avg_price: position.avg_price.to_string(),
            current_value: position.current_value.to_string(),
            cash_pnl: position.cash_pnl.to_string(),
            percent_pnl: position.percent_pnl.to_string(),
            title: position.title,
            outcome: position.outcome,
            redeemable: position.redeemable,
            mergeable: position.mergeable,
        }
    }

    pub fn map_activity(activity: SdkActivity) -> Activity {
        Activity {
            timestamp: activity.timestamp,
            activity_type: format!("{:?}", activity.activity_type),
            condition_id: activity.condition_id.map(|id| id.to_string()),
            size: activity.size.to_string(),
            usdc_size: activity.usdc_size.to_string(),
            transaction_hash: activity.transaction_hash.to_string(),
            title: activity.title,
            outcome: activity.outcome,
        }
    }

    pub fn map_portfolio_value(value: SdkValue) -> PortfolioValue {
        PortfolioValue {
            user: value.user.to_string(),
            value: value.value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_cursor_round_trip() {
        let encoded = encode_offset_cursor(OffsetCursorState {
            offset: 40,
            page_size: 20,
        });
        assert_eq!(encoded.as_str(), "offset:40:page_size:20");
        let decoded = decode_offset_cursor(Some(&encoded), 20).unwrap();
        assert_eq!(decoded.offset, 40);
        assert_eq!(decoded.page_size, 20);
    }
}
