//! API bindings and normalized models for Polymarket services.

#![deny(unsafe_code)]

pub mod clob;
mod de;
pub mod gamma;
pub mod shared;

pub use shared::{OrderSide, OrderType};
