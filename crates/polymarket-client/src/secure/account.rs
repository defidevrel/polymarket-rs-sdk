//! Authenticated account reads (CLOB + data API defaults).

use std::str::FromStr as _;

use polymarket_client_sdk_v2::clob::types::request::TradesRequest;
use polymarket_client_sdk_v2::clob::types::response::{
    CurrentRewardResponse, NotificationResponse, TradeResponse,
};
use polymarket_client_sdk_v2::types::{B256, U256};

use crate::account::{
    FetchPortfolioValueError, FetchPortfolioValueRequest, ListActivityError, ListActivityRequest,
    ListPositionsError, ListPositionsRequest, PortfolioValue,
};
use crate::account_client::{ListActivityPaginator, ListPositionsPaginator};
use crate::error::{user_input, UserInputError};
use crate::secure::secure_client::SecureClient;

macro_rules! secure_account_error {
    ($name:ident) => {
        #[derive(Debug, thiserror::Error, Clone)]
        pub enum $name {
            #[error(transparent)]
            UserInput(#[from] UserInputError),
            #[error("SDK error: {0}")]
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

secure_account_error!(ListAccountTradesError);
secure_account_error!(FetchNotificationsError);
secure_account_error!(FetchOrderScoringError);
secure_account_error!(ListCurrentRewardsError);

#[derive(Clone, Debug, Default)]
pub struct ListAccountTradesRequest {
    pub token_id: Option<String>,
    pub market: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AccountTrade {
    pub trade_id: String,
    pub token_id: String,
    pub market: String,
    pub side: String,
    pub price: String,
    pub size: String,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub notification_type: u32,
    pub order_id: String,
    pub token_id: String,
    pub market: String,
    pub side: String,
    pub price: String,
    pub matched_size: String,
}

#[derive(Clone, Debug)]
pub struct CurrentReward {
    pub condition_id: String,
    pub max_spread: String,
    pub min_size: String,
}

#[derive(Clone, Debug)]
pub struct FetchOrderScoringRequest {
    pub order_id: String,
}

impl SecureClient {
    pub fn list_positions(
        &self,
        request: ListPositionsRequest,
    ) -> Result<ListPositionsPaginator, ListPositionsError> {
        self.public().list_positions(with_wallet(self, request))
    }

    pub async fn fetch_portfolio_value(
        &self,
        request: FetchPortfolioValueRequest,
    ) -> Result<Vec<PortfolioValue>, FetchPortfolioValueError> {
        self.public()
            .fetch_portfolio_value(with_wallet(self, request))
            .await
    }

    pub fn list_activity(
        &self,
        request: ListActivityRequest,
    ) -> Result<ListActivityPaginator, ListActivityError> {
        self.public().list_activity(with_wallet(self, request))
    }

    pub async fn list_account_trades(
        &self,
        request: ListAccountTradesRequest,
    ) -> Result<Vec<AccountTrade>, ListAccountTradesError> {
        let mut req = TradesRequest::default();
        if let Some(token_id) = request.token_id {
            req.asset_id = Some(parse_token_id(&token_id)?);
        }
        if let Some(market) = request.market {
            req.market = Some(parse_market_id(&market)?);
        }

        let mut cursor = None;
        let mut all = Vec::new();
        loop {
            let page = self
                .clob
                .trades(&req, cursor.clone())
                .await
                .map_err(|e| ListAccountTradesError::Sdk(e.to_string()))?;
            all.extend(page.data.into_iter().map(map_account_trade));
            if page.next_cursor.is_empty() || page.next_cursor == "LTE=" {
                break;
            }
            cursor = Some(page.next_cursor);
        }
        Ok(all)
    }

    pub async fn fetch_notifications(&self) -> Result<Vec<Notification>, FetchNotificationsError> {
        let notifications = self
            .clob
            .notifications()
            .await
            .map_err(|e| FetchNotificationsError::Sdk(e.to_string()))?;
        Ok(notifications.into_iter().map(map_notification).collect())
    }

    pub async fn fetch_order_scoring(
        &self,
        request: FetchOrderScoringRequest,
    ) -> Result<bool, FetchOrderScoringError> {
        if request.order_id.trim().is_empty() {
            return Err(FetchOrderScoringError::UserInput(user_input(
                "order_id cannot be empty",
            )));
        }
        let response = self
            .clob
            .is_order_scoring(&request.order_id)
            .await
            .map_err(|e| FetchOrderScoringError::Sdk(e.to_string()))?;
        Ok(response.scoring)
    }

    pub async fn list_current_rewards(
        &self,
    ) -> Result<Vec<CurrentReward>, ListCurrentRewardsError> {
        let mut cursor = None;
        let mut all = Vec::new();
        loop {
            let page = self
                .clob
                .current_rewards(cursor.clone())
                .await
                .map_err(|e| ListCurrentRewardsError::Sdk(e.to_string()))?;
            all.extend(page.data.into_iter().map(map_current_reward));
            if page.next_cursor.is_empty() || page.next_cursor == "LTE=" {
                break;
            }
            cursor = Some(page.next_cursor);
        }
        Ok(all)
    }
}

fn with_wallet<T>(client: &SecureClient, mut request: T) -> T
where
    T: DefaultWallet,
{
    if request.user().trim().is_empty() {
        request.set_user(client.wallet().to_string());
    }
    request
}

trait DefaultWallet {
    fn user(&self) -> &str;
    fn set_user(&mut self, user: String);
}

impl DefaultWallet for ListPositionsRequest {
    fn user(&self) -> &str {
        &self.user
    }
    fn set_user(&mut self, user: String) {
        self.user = user;
    }
}

impl DefaultWallet for FetchPortfolioValueRequest {
    fn user(&self) -> &str {
        &self.user
    }
    fn set_user(&mut self, user: String) {
        self.user = user;
    }
}

impl DefaultWallet for ListActivityRequest {
    fn user(&self) -> &str {
        &self.user
    }
    fn set_user(&mut self, user: String) {
        self.user = user;
    }
}

fn parse_token_id(token_id: &str) -> Result<U256, UserInputError> {
    U256::from_str(token_id).map_err(|e| user_input(format!("invalid token_id: {e}")))
}

fn parse_market_id(market: &str) -> Result<B256, UserInputError> {
    B256::from_str(market).map_err(|e| user_input(format!("invalid market id: {e}")))
}

fn map_account_trade(trade: TradeResponse) -> AccountTrade {
    AccountTrade {
        trade_id: trade.id,
        token_id: trade.asset_id.to_string(),
        market: trade.market.to_string(),
        side: format!("{:?}", trade.side),
        price: trade.price.to_string(),
        size: trade.size.to_string(),
        status: format!("{:?}", trade.status),
    }
}

fn map_notification(notification: NotificationResponse) -> Notification {
    Notification {
        notification_type: notification.r#type,
        order_id: notification.payload.order_id,
        token_id: notification.payload.asset_id.to_string(),
        market: notification.payload.market.to_string(),
        side: format!("{:?}", notification.payload.side),
        price: notification.payload.price.to_string(),
        matched_size: notification.payload.matched_size.to_string(),
    }
}

fn map_current_reward(reward: CurrentRewardResponse) -> CurrentReward {
    CurrentReward {
        condition_id: reward.condition_id.to_string(),
        max_spread: reward.rewards_max_spread.to_string(),
        min_size: reward.rewards_min_size.to_string(),
    }
}
