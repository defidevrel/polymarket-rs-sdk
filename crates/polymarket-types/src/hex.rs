use crate::error::ValidationError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A string encoded as hexadecimal and prefixed with `0x`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HexString(String);

impl HexString {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HexString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An EVM account or contract address.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EvmAddress(HexString);

impl EvmAddress {
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for EvmAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Returns `true` when `value` is a hex string prefixed with `0x`.
#[must_use]
pub fn is_hex_string(value: &str) -> bool {
    value.starts_with("0x") && value[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Returns `true` when two EVM addresses are equal, ignoring checksum casing.
#[must_use]
pub fn is_same_evm_address(left: &EvmAddress, right: &EvmAddress) -> bool {
    left.as_str().eq_ignore_ascii_case(right.as_str())
}

/// Parse and validate an EVM address.
pub fn parse_evm_address(value: &str) -> Result<EvmAddress, ValidationError> {
    if !is_hex_string(value) {
        return Err(ValidationError::new(format!(
            "expected hex address prefixed with 0x, received: {value}"
        )));
    }
    if value.len() != 42 {
        return Err(ValidationError::new(format!(
            "expected 20-byte address (42 chars), received length {}",
            value.len()
        )));
    }
    Ok(EvmAddress(HexString(value.to_string())))
}

impl FromStr for EvmAddress {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_evm_address(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_address() {
        let addr = parse_evm_address("0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB").unwrap();
        assert_eq!(addr.as_str(), "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB");
    }

    #[test]
    fn rejects_invalid_address() {
        assert!(parse_evm_address("not-an-address").is_err());
    }
}
