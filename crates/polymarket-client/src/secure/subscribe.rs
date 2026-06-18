//! SecureClient realtime subscriptions (includes user channel).

use crate::secure::secure_client::SecureClient;
use crate::subscriptions::{
    merge_streams, subscribe_one, subscribe_user, SubscribeError, SubscriptionHandle,
    SubscriptionSpec,
};

impl SecureClient {
    /// Subscribe to realtime channels, including authenticated user events.
    pub fn subscribe(
        &self,
        specs: Vec<SubscriptionSpec>,
    ) -> Result<SubscriptionHandle, SubscribeError> {
        if specs.is_empty() {
            return Err(SubscribeError::UserInput(crate::error::user_input(
                "at least one subscription spec is required",
            )));
        }

        let ws = &self.public().ws;
        let mut streams = Vec::with_capacity(specs.len());

        for spec in specs {
            let stream = match spec {
                SubscriptionSpec::User(user) => subscribe_user(
                    ws,
                    self.credentials().to_sdk_credentials().map_err(|e| {
                        SubscribeError::Transport(format!("invalid credentials: {e}"))
                    })?,
                    self.wallet(),
                    user,
                )?,
                other => subscribe_one(ws, other)?,
            };
            streams.push(stream);
        }

        Ok(SubscriptionHandle::new(merge_streams(streams), None))
    }
}
