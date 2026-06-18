use crate::error::ValidationError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn parse(value: impl Into<String>) -> Result<Self, ValidationError> {
                let value = value.into();
                if value.is_empty() {
                    return Err(ValidationError::new(format!(
                        "{} cannot be empty",
                        stringify!($name)
                    )));
                }
                Ok(Self(value))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = ValidationError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::parse(s)
            }
        }
    };
}

string_id!(MarketId);
string_id!(EventId);
string_id!(TokenId);
string_id!(PaginationCursor);
string_id!(DecimalString);

/// CTF condition identifier (bytes32 hex).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CtfConditionId(String);

impl CtfConditionId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ValidationError> {
        let value = value.as_ref();
        if !value.starts_with("0x") || value.len() != 66 {
            return Err(ValidationError::new(format!(
                "expected 32-byte condition id, received: {value}"
            )));
        }
        Ok(Self(value.to_string()))
    }
}

impl fmt::Display for CtfConditionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for CtfConditionId {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
