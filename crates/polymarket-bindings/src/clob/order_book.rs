use polymarket_types::{CtfConditionId, DecimalString, TokenId};
use serde::Deserialize;

use crate::de::deserialize_decimalish;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct OrderBookLevel {
    #[serde(deserialize_with = "deserialize_decimalish")]
    pub price: Option<String>,
    #[serde(deserialize_with = "deserialize_decimalish")]
    pub size: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderBook {
    pub market: CtfConditionId,
    pub token_id: TokenId,
    pub timestamp: Option<i64>,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub min_order_size: DecimalString,
    pub tick_size: DecimalString,
    pub neg_risk: bool,
    pub last_trade_price: Option<DecimalString>,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
struct OrderBookRaw {
    market: String,
    asset_id: String,
    timestamp: Option<String>,
    bids: Vec<OrderBookLevel>,
    asks: Vec<OrderBookLevel>,
    #[serde(rename = "min_order_size", deserialize_with = "deserialize_decimalish")]
    min_order_size: Option<String>,
    #[serde(rename = "tick_size", deserialize_with = "deserialize_decimalish")]
    tick_size: Option<String>,
    neg_risk: bool,
    #[serde(rename = "last_trade_price", default, deserialize_with = "deserialize_decimalish")]
    last_trade_price: Option<String>,
    hash: String,
}

impl OrderBook {
    fn from_raw(raw: OrderBookRaw) -> Result<Self, String> {
        let market = CtfConditionId::parse(raw.market).map_err(|e| e.message)?;
        let token_id = TokenId::parse(raw.asset_id).map_err(|e| e.message)?;
        let timestamp = raw.timestamp.and_then(|t| t.parse().ok());
        let min_order_size = raw
            .min_order_size
            .and_then(|v| DecimalString::parse(v).ok())
            .ok_or_else(|| "missing min_order_size".to_string())?;
        let tick_size = raw
            .tick_size
            .and_then(|v| DecimalString::parse(v).ok())
            .ok_or_else(|| "missing tick_size".to_string())?;
        let last_trade_price = raw
            .last_trade_price
            .and_then(|v| DecimalString::parse(v).ok());

        Ok(Self {
            market,
            token_id,
            timestamp,
            bids: raw.bids,
            asks: raw.asks,
            min_order_size,
            tick_size,
            neg_risk: raw.neg_risk,
            last_trade_price,
            hash: raw.hash,
        })
    }
}

impl<'de> Deserialize<'de> for OrderBook {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = OrderBookRaw::deserialize(deserializer)?;
        Self::from_raw(raw).map_err(serde::de::Error::custom)
    }
}
