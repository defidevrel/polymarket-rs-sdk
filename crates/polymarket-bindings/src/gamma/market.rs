use polymarket_types::{CtfConditionId, DecimalString, EventId, EvmAddress, MarketId, TokenId};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::de::{
    deserialize_decimalish, deserialize_empty_string_as_none, deserialize_string_array,
};

/// Reference to an event embedded in a market.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketEventRef {
    pub id: polymarket_types::EventId,
    pub slug: Option<String>,
    pub title: Option<String>,
}

/// Tag reference attached to a market or event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TagReference {
    pub id: String,
    pub slug: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketState {
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub accepting_orders: Option<bool>,
    pub enable_order_book: Option<bool>,
    pub neg_risk: Option<bool>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub closed_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketOutcome {
    pub label: String,
    pub token_id: Option<TokenId>,
    pub price: Option<DecimalString>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketOutcomes {
    pub yes: MarketOutcome,
    pub no: MarketOutcome,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketMetrics {
    pub volume: Option<DecimalString>,
    pub volume_num: Option<DecimalString>,
    pub volume24hr: Option<DecimalString>,
    pub liquidity: Option<DecimalString>,
    pub liquidity_num: Option<DecimalString>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketPrices {
    pub best_bid: Option<DecimalString>,
    pub best_ask: Option<DecimalString>,
    pub last_trade_price: Option<DecimalString>,
    pub spread: Option<DecimalString>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketTrading {
    pub minimum_order_size: Option<DecimalString>,
    pub minimum_tick_size: Option<DecimalString>,
    pub seconds_delay: Option<i64>,
    pub fees_enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketResolution {
    pub question_id: Option<String>,
    pub uma_resolution_status: Option<String>,
    pub source: Option<String>,
    pub resolved_by: Option<EvmAddress>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Market {
    pub id: MarketId,
    pub slug: Option<String>,
    pub condition_id: Option<CtfConditionId>,
    pub question: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub state: MarketState,
    pub outcomes: MarketOutcomes,
    pub metrics: MarketMetrics,
    pub prices: MarketPrices,
    pub trading: MarketTrading,
    pub resolution: MarketResolution,
    pub events: Vec<MarketEventRef>,
    pub tags: Vec<TagReference>,
}

/// Raw Gamma API market payload (snake_case fields).
#[derive(Debug, Deserialize)]
pub struct GammaMarket {
    pub id: String,
    pub slug: Option<String>,
    #[serde(rename = "conditionId")]
    pub condition_id: Option<String>,
    pub question: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    #[serde(rename = "acceptingOrders")]
    pub accepting_orders: Option<bool>,
    #[serde(rename = "enableOrderBook")]
    pub enable_order_book: Option<bool>,
    #[serde(rename = "negRisk")]
    pub neg_risk: Option<bool>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    #[serde(rename = "closedTime")]
    pub closed_time: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_array")]
    pub outcomes: Vec<String>,
    #[serde(
        default,
        rename = "outcomePrices",
        deserialize_with = "deserialize_string_array"
    )]
    pub outcome_prices: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_decimalish")]
    pub volume: Option<String>,
    #[serde(default, rename = "volumeNum", deserialize_with = "deserialize_decimalish")]
    pub volume_num: Option<String>,
    #[serde(default, rename = "volume24hr", deserialize_with = "deserialize_decimalish")]
    pub volume24hr: Option<String>,
    #[serde(default, deserialize_with = "deserialize_decimalish")]
    pub liquidity: Option<String>,
    #[serde(default, rename = "liquidityNum", deserialize_with = "deserialize_decimalish")]
    pub liquidity_num: Option<String>,
    #[serde(default, rename = "bestBid", deserialize_with = "deserialize_decimalish")]
    pub best_bid: Option<String>,
    #[serde(default, rename = "bestAsk", deserialize_with = "deserialize_decimalish")]
    pub best_ask: Option<String>,
    #[serde(default, rename = "lastTradePrice", deserialize_with = "deserialize_decimalish")]
    pub last_trade_price: Option<String>,
    #[serde(default, deserialize_with = "deserialize_decimalish")]
    pub spread: Option<String>,
    #[serde(default, rename = "orderMinSize", deserialize_with = "deserialize_decimalish")]
    pub order_min_size: Option<String>,
    #[serde(
        default,
        rename = "orderPriceMinTickSize",
        deserialize_with = "deserialize_decimalish"
    )]
    pub order_price_min_tick_size: Option<String>,
    #[serde(rename = "secondsDelay")]
    pub seconds_delay: Option<i64>,
    #[serde(rename = "feesEnabled")]
    pub fees_enabled: Option<bool>,
    #[serde(rename = "questionID")]
    pub question_id: Option<String>,
    #[serde(rename = "umaResolutionStatus")]
    pub uma_resolution_status: Option<String>,
    #[serde(rename = "resolutionSource")]
    pub resolution_source: Option<String>,
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub resolved_by: Option<String>,
    #[serde(
        default,
        rename = "clobTokenIds",
        deserialize_with = "deserialize_string_array"
    )]
    pub clob_token_ids: Vec<String>,
    #[serde(default)]
    pub(crate) events: Vec<GammaMarketEvent>,
    #[serde(default)]
    pub tags: Vec<TagReference>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GammaMarketEvent {
    id: String,
    slug: Option<String>,
    title: Option<String>,
}

pub const fn has_binary_outcomes(market: &GammaMarket) -> bool {
    market.outcomes.len() == 2
}

pub fn try_normalize_market(raw: GammaMarket) -> Result<Market, String> {
    if !has_binary_outcomes(&raw) {
        return Err(format!(
            "expected binary market outcomes, received {}",
            raw.outcomes.len()
        ));
    }

    let id = MarketId::parse(raw.id).map_err(|e| e.message)?;
    let condition_id = raw
        .condition_id
        .as_deref()
        .map(CtfConditionId::parse)
        .transpose()
        .map_err(|e| e.message)?;

    let resolved_by = raw
        .resolved_by
        .as_deref()
        .map(EvmAddress::from_str)
        .transpose()
        .map_err(|e| e.message)?;

    let parse_decimal = |s: Option<String>| {
        s.filter(|v| !v.is_empty())
            .and_then(|v| DecimalString::parse(v).ok())
    };

    let yes_label = raw.outcomes[0].clone();
    let no_label = raw.outcomes[1].clone();

    Ok(Market {
        id,
        slug: raw.slug,
        condition_id,
        question: raw.question,
        description: raw.description,
        category: raw.category,
        image: raw.image,
        icon: raw.icon,
        state: MarketState {
            active: raw.active,
            closed: raw.closed,
            archived: raw.archived,
            accepting_orders: raw.accepting_orders,
            enable_order_book: raw.enable_order_book,
            neg_risk: raw.neg_risk,
            start_date: raw.start_date,
            end_date: raw.end_date,
            closed_time: raw.closed_time,
        },
        outcomes: MarketOutcomes {
            yes: MarketOutcome {
                label: yes_label,
                token_id: raw
                    .clob_token_ids
                    .first()
                    .and_then(|t| TokenId::parse(t.clone()).ok()),
                price: raw
                    .outcome_prices
                    .first()
                    .and_then(|p| DecimalString::parse(p.clone()).ok()),
            },
            no: MarketOutcome {
                label: no_label,
                token_id: raw
                    .clob_token_ids
                    .get(1)
                    .and_then(|t| TokenId::parse(t.clone()).ok()),
                price: raw
                    .outcome_prices
                    .get(1)
                    .and_then(|p| DecimalString::parse(p.clone()).ok()),
            },
        },
        metrics: MarketMetrics {
            volume: parse_decimal(raw.volume),
            volume_num: parse_decimal(raw.volume_num),
            volume24hr: parse_decimal(raw.volume24hr),
            liquidity: parse_decimal(raw.liquidity),
            liquidity_num: parse_decimal(raw.liquidity_num),
        },
        prices: MarketPrices {
            best_bid: parse_decimal(raw.best_bid),
            best_ask: parse_decimal(raw.best_ask),
            last_trade_price: parse_decimal(raw.last_trade_price),
            spread: parse_decimal(raw.spread),
        },
        trading: MarketTrading {
            minimum_order_size: parse_decimal(raw.order_min_size),
            minimum_tick_size: parse_decimal(raw.order_price_min_tick_size),
            seconds_delay: raw.seconds_delay,
            fees_enabled: raw.fees_enabled,
        },
        resolution: MarketResolution {
            question_id: raw.question_id,
            uma_resolution_status: raw.uma_resolution_status,
            source: raw.resolution_source,
            resolved_by,
        },
        events: raw
            .events
            .into_iter()
            .filter_map(|e| {
                Some(MarketEventRef {
                    id: EventId::parse(e.id).ok()?,
                    slug: e.slug,
                    title: e.title,
                })
            })
            .collect(),
        tags: raw.tags,
    })
}

pub fn normalize_market(raw: GammaMarket) -> Market {
    try_normalize_market(raw).expect("binary market normalization should succeed")
}

#[derive(Debug, Deserialize)]
pub struct ListMarketsKeysetRaw {
    pub markets: Vec<GammaMarket>,
    pub next_cursor: Option<String>,
}

#[derive(Debug)]
pub struct ListMarketsKeysetResponse {
    pub items: Vec<Market>,
    pub next_cursor: Option<polymarket_types::PaginationCursor>,
}

impl ListMarketsKeysetResponse {
    pub fn from_raw(raw: ListMarketsKeysetRaw) -> Self {
        let items = raw
            .markets
            .into_iter()
            .filter_map(|m| try_normalize_market(m).ok())
            .collect();
        let next_cursor = raw
            .next_cursor
            .and_then(|c| polymarket_types::PaginationCursor::parse(c).ok());
        Self { items, next_cursor }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_binary_market() {
        let raw = GammaMarket {
            id: "123".into(),
            slug: Some("test-market".into()),
            condition_id: Some(
                "0x4cd77d456c83e7d8c569a8fb8f6396c3f40154f657e6d970733e2b1b6a7110ff".into(),
            ),
            question: Some("Will it happen?".into()),
            description: None,
            category: Some("politics".into()),
            image: None,
            icon: None,
            active: Some(true),
            closed: Some(false),
            archived: Some(false),
            accepting_orders: Some(true),
            enable_order_book: Some(true),
            neg_risk: Some(false),
            start_date: None,
            end_date: None,
            closed_time: None,
            outcomes: vec!["Yes".into(), "No".into()],
            outcome_prices: vec!["0.52".into(), "0.48".into()],
            volume: Some("1000".into()),
            volume_num: None,
            volume24hr: None,
            liquidity: None,
            liquidity_num: None,
            best_bid: Some("0.51".into()),
            best_ask: Some("0.53".into()),
            last_trade_price: Some("0.52".into()),
            spread: Some("0.02".into()),
            order_min_size: Some("5".into()),
            order_price_min_tick_size: Some("0.01".into()),
            seconds_delay: None,
            fees_enabled: Some(false),
            question_id: None,
            uma_resolution_status: None,
            resolution_source: None,
            resolved_by: None,
            clob_token_ids: vec!["token-yes".into(), "token-no".into()],
            events: vec![],
            tags: vec![],
        };

        let market = normalize_market(raw);
        assert_eq!(market.id.as_str(), "123");
        assert_eq!(market.outcomes.yes.label, "Yes");
        assert_eq!(
            market.outcomes.yes.token_id.as_ref().map(|t| t.as_str()),
            Some("token-yes")
        );
    }

    #[test]
    fn skips_non_binary_in_list_response() {
        let raw = ListMarketsKeysetRaw {
            markets: vec![GammaMarket {
                id: "1".into(),
                slug: None,
                condition_id: None,
                question: None,
                description: None,
                category: None,
                image: None,
                icon: None,
                active: None,
                closed: None,
                archived: None,
                accepting_orders: None,
                enable_order_book: None,
                neg_risk: None,
                start_date: None,
                end_date: None,
                closed_time: None,
                outcomes: vec!["A".into(), "B".into(), "C".into()],
                outcome_prices: vec![],
                volume: None,
                volume_num: None,
                volume24hr: None,
                liquidity: None,
                liquidity_num: None,
                best_bid: None,
                best_ask: None,
                last_trade_price: None,
                spread: None,
                order_min_size: None,
                order_price_min_tick_size: None,
                seconds_delay: None,
                fees_enabled: None,
                question_id: None,
                uma_resolution_status: None,
                resolution_source: None,
                resolved_by: None,
                clob_token_ids: vec![],
                events: vec![],
                tags: vec![],
            }],
            next_cursor: None,
        };

        let response = ListMarketsKeysetResponse::from_raw(raw);
        assert!(response.items.is_empty());
    }
}
