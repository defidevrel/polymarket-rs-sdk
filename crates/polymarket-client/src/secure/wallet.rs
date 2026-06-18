//! On-chain wallet operations (CTF split/merge/redeem).

use std::str::FromStr as _;

use alloy::providers::ProviderBuilder;
use polymarket_client_sdk_v2::ctf::types::{
    MergePositionsRequest as SdkMergeRequest, RedeemPositionsRequest as SdkRedeemRequest,
    SplitPositionRequest as SdkSplitRequest,
};
use polymarket_client_sdk_v2::ctf::Client as CtfClient;
use polymarket_client_sdk_v2::types::{Address, B256, U256};
use polymarket_client_sdk_v2::POLYGON;

use crate::error::{user_input, UserInputError};
use crate::secure::secure_client::SecureClient;

#[derive(Debug, thiserror::Error, Clone)]
pub enum WalletOperationError {
    #[error(transparent)]
    UserInput(#[from] UserInputError),
    #[error("on-chain error: {0}")]
    OnChain(String),
}

#[derive(Clone, Debug)]
pub struct TransactionOutcome {
    pub transaction_hash: String,
    pub block_number: u64,
}

#[derive(Clone, Debug)]
pub struct SplitPositionRequest {
    pub condition_id: String,
    /// Amount in collateral base units (USDC has 6 decimals).
    pub amount: u128,
}

#[derive(Clone, Debug)]
pub struct MergePositionsRequest {
    pub condition_id: String,
    pub amount: u128,
}

#[derive(Clone, Debug, Default)]
pub struct RedeemPositionsRequest {
    pub condition_id: String,
    /// Index sets to redeem. Empty uses the binary market default.
    pub index_sets: Vec<u64>,
}

impl SecureClient {
    pub async fn split_position(
        &self,
        request: SplitPositionRequest,
    ) -> Result<TransactionOutcome, WalletOperationError> {
        validate_amount(request.amount)?;
        let client = self.ctf_client().await?;
        let condition_id = parse_condition_id(&request.condition_id)?;
        let collateral = collateral_token(self)?;

        let sdk_request = SdkSplitRequest::for_binary_market(
            collateral,
            condition_id,
            U256::from(request.amount),
        );

        let response = client
            .split_position(&sdk_request)
            .await
            .map_err(|e| WalletOperationError::OnChain(e.to_string()))?;

        Ok(TransactionOutcome {
            transaction_hash: response.transaction_hash.to_string(),
            block_number: response.block_number,
        })
    }

    pub async fn merge_positions(
        &self,
        request: MergePositionsRequest,
    ) -> Result<TransactionOutcome, WalletOperationError> {
        validate_amount(request.amount)?;
        let client = self.ctf_client().await?;
        let condition_id = parse_condition_id(&request.condition_id)?;
        let collateral = collateral_token(self)?;

        let sdk_request = SdkMergeRequest::for_binary_market(
            collateral,
            condition_id,
            U256::from(request.amount),
        );

        let response = client
            .merge_positions(&sdk_request)
            .await
            .map_err(|e| WalletOperationError::OnChain(e.to_string()))?;

        Ok(TransactionOutcome {
            transaction_hash: response.transaction_hash.to_string(),
            block_number: response.block_number,
        })
    }

    pub async fn redeem_positions(
        &self,
        request: RedeemPositionsRequest,
    ) -> Result<TransactionOutcome, WalletOperationError> {
        let client = self.ctf_client().await?;
        let condition_id = parse_condition_id(&request.condition_id)?;
        let collateral = collateral_token(self)?;

        let sdk_request = SdkRedeemRequest::for_binary_market(collateral, condition_id);

        let response = client
            .redeem_positions(&sdk_request)
            .await
            .map_err(|e| WalletOperationError::OnChain(e.to_string()))?;

        Ok(TransactionOutcome {
            transaction_hash: response.transaction_hash.to_string(),
            block_number: response.block_number,
        })
    }

    async fn ctf_client(
        &self,
    ) -> Result<CtfClient<impl alloy::providers::Provider + Clone>, WalletOperationError> {
        let provider = ProviderBuilder::new()
            .wallet(self.signer.clone())
            .connect(self.environment().rpc)
            .await
            .map_err(|e| WalletOperationError::OnChain(e.to_string()))?;

        CtfClient::new(provider, POLYGON).map_err(|e| WalletOperationError::OnChain(e.to_string()))
    }
}

fn collateral_token(client: &SecureClient) -> Result<Address, WalletOperationError> {
    Address::from_str(client.environment().collateral_token.as_str()).map_err(|e| {
        WalletOperationError::UserInput(user_input(format!("invalid collateral token: {e}")))
    })
}

fn parse_condition_id(value: &str) -> Result<B256, UserInputError> {
    B256::from_str(value).map_err(|e| user_input(format!("invalid condition_id: {e}")))
}

fn validate_amount(amount: u128) -> Result<(), UserInputError> {
    if amount == 0 {
        return Err(user_input("amount must be greater than zero"));
    }
    Ok(())
}
