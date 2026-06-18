use thiserror::Error;

/// Top-level SDK error.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ListMarkets(#[from] ListMarketsError),
    #[error(transparent)]
    FetchMarket(#[from] FetchMarketError),
    #[error(transparent)]
    ListEvents(#[from] ListEventsError),
    #[error(transparent)]
    FetchMidpoint(#[from] FetchMidpointError),
    #[error(transparent)]
    FetchOrderBook(#[from] FetchOrderBookError),
}

#[derive(Debug, Error, Clone)]
#[error("invalid input: {0}")]
pub struct UserInputError(pub String);

#[derive(Debug, Error, Clone, Copy)]
#[error("rate limit exceeded")]
pub struct RateLimitError;

#[derive(Debug, Error, Clone)]
#[error("request rejected with status {status}: {message}")]
pub struct RequestRejectedError {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Error, Clone)]
#[error("transport error: {0}")]
pub struct TransportError(pub String);

#[derive(Debug, Error, Clone)]
#[error("unexpected response: {0}")]
pub struct UnexpectedResponseError(pub String);

macro_rules! action_error {
    ($name:ident) => {
        #[derive(Debug, Error, Clone)]
        pub enum $name {
            #[error(transparent)]
            UserInput(#[from] UserInputError),
            #[error(transparent)]
            RateLimit(#[from] RateLimitError),
            #[error(transparent)]
            RequestRejected(#[from] RequestRejectedError),
            #[error(transparent)]
            Transport(#[from] TransportError),
            #[error(transparent)]
            UnexpectedResponse(#[from] UnexpectedResponseError),
        }

        impl $name {
            #[must_use]
            pub fn is_error(err: &(dyn std::error::Error + 'static)) -> bool {
                err.downcast_ref::<Self>().is_some()
                    || err.downcast_ref::<UserInputError>().is_some()
                    || err.downcast_ref::<RateLimitError>().is_some()
                    || err.downcast_ref::<RequestRejectedError>().is_some()
                    || err.downcast_ref::<TransportError>().is_some()
                    || err.downcast_ref::<UnexpectedResponseError>().is_some()
            }
        }
    };
}

action_error!(ListMarketsError);
action_error!(FetchMarketError);
action_error!(ListEventsError);
action_error!(FetchMidpointError);
action_error!(FetchOrderBookError);

impl From<reqwest::Error> for TransportError {
    fn from(value: reqwest::Error) -> Self {
        Self(value.to_string())
    }
}

pub fn user_input(message: impl Into<String>) -> UserInputError {
    UserInputError(message.into())
}

pub fn unexpected_response(message: impl Into<String>) -> UnexpectedResponseError {
    UnexpectedResponseError(message.into())
}
