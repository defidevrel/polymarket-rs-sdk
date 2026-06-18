use polymarket_types::EventId;
use serde::{Deserialize, Serialize};

use super::market::{try_normalize_market, GammaMarket, Market, TagReference};

/// Normalized Polymarket event.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Event {
    pub id: EventId,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub state: EventState,
    pub markets: Vec<Market>,
    pub tags: Vec<TagReference>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventState {
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub featured: Option<bool>,
    pub restricted: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GammaEvent {
    pub id: String,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub featured: Option<bool>,
    pub restricted: Option<bool>,
    #[serde(default)]
    pub markets: Vec<GammaMarket>,
    #[serde(default)]
    pub tags: Vec<TagReference>,
}

pub fn try_normalize_event(raw: GammaEvent) -> Result<Event, String> {
    let id = EventId::parse(raw.id).map_err(|e| e.message)?;
    let markets = raw
        .markets
        .into_iter()
        .filter_map(|m| try_normalize_market(m).ok())
        .collect();

    Ok(Event {
        id,
        slug: raw.slug,
        title: raw.title,
        subtitle: raw.subtitle,
        description: raw.description,
        category: raw.category,
        subcategory: raw.subcategory,
        image: raw.image,
        icon: raw.icon,
        state: EventState {
            active: raw.active,
            closed: raw.closed,
            archived: raw.archived,
            featured: raw.featured,
            restricted: raw.restricted,
        },
        markets,
        tags: raw.tags,
    })
}

pub fn normalize_event(raw: GammaEvent) -> Event {
    try_normalize_event(raw).expect("event normalization should succeed for valid gamma events")
}

#[derive(Debug, Deserialize)]
pub struct ListEventsKeysetRaw {
    pub events: Vec<GammaEvent>,
    pub next_cursor: Option<String>,
}

#[derive(Debug)]
pub struct ListEventsKeysetResponse {
    pub items: Vec<Event>,
    pub next_cursor: Option<polymarket_types::PaginationCursor>,
}

impl ListEventsKeysetResponse {
    pub fn from_raw(raw: ListEventsKeysetRaw) -> Self {
        let items = raw.events.into_iter().map(normalize_event).collect();
        let next_cursor = raw
            .next_cursor
            .and_then(|c| polymarket_types::PaginationCursor::parse(c).ok());
        Self { items, next_cursor }
    }
}
