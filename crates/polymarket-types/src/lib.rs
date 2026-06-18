//! Shared primitives for the Polymarket Rust SDK.

#![deny(unsafe_code)]

mod error;
mod hex;
mod ids;

pub use error::{PolymarketError, ValidationError};
pub use hex::{is_hex_string, is_same_evm_address, parse_evm_address, EvmAddress, HexString};
pub use ids::{CtfConditionId, DecimalString, EventId, MarketId, PaginationCursor, TokenId};
