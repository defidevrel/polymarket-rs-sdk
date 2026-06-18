use polymarket_client_sdk_v2::auth::Credentials;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use uuid::Uuid;

/// Persisted L2 API credentials for session reuse.
///
/// Store securely — the secret and passphrase grant trading access.
#[derive(Clone, Debug, Deserialize)]
pub struct ApiCredentials {
    pub key: String,
    secret: SecretString,
    passphrase: SecretString,
}

impl ApiCredentials {
    pub fn new(key: impl Into<String>, secret: impl Into<String>, passphrase: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            secret: SecretString::from(secret.into()),
            passphrase: SecretString::from(passphrase.into()),
        }
    }

    pub fn key_uuid(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.key)
    }

    pub fn secret(&self) -> &SecretString {
        &self.secret
    }

    pub fn passphrase(&self) -> &SecretString {
        &self.passphrase
    }

    pub(crate) fn to_sdk_credentials(&self) -> Result<Credentials, uuid::Error> {
        Ok(Credentials::new(
            self.key_uuid()?,
            self.secret.expose_secret().to_string(),
            self.passphrase.expose_secret().to_string(),
        ))
    }

    pub(crate) fn from_sdk(credentials: &Credentials) -> Self {
        Self {
            key: credentials.key().to_string(),
            secret: credentials.secret().clone(),
            passphrase: credentials.passphrase().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_key() {
        let id = Uuid::new_v4();
        let creds = ApiCredentials::new(id.to_string(), "secret", "pass");
        assert_eq!(creds.key_uuid().unwrap(), id);
    }
}
