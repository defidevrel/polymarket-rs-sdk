//! API bindings and normalized models for Polymarket services.

#![deny(unsafe_code)]

mod de;
pub mod clob;
pub mod gamma;
pub mod shared;

pub use shared::{OrderSide, OrderType};
