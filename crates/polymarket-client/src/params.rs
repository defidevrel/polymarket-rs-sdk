pub fn append_query(out: &mut Vec<(String, String)>, key: &str, value: &(impl ToString + ?Sized)) {
    out.push((key.to_string(), value.to_string()));
}

pub fn append_optional(
    out: &mut Vec<(String, String)>,
    key: &str,
    value: Option<&(impl ToString + ?Sized)>,
) {
    if let Some(v) = value {
        append_query(out, key, v);
    }
}

pub fn append_bool(out: &mut Vec<(String, String)>, key: &str, value: Option<bool>) {
    if let Some(v) = value {
        append_query(out, key, &v);
    }
}

pub fn append_string_slice(out: &mut Vec<(String, String)>, key: &str, values: Option<&[String]>) {
    if let Some(values) = values {
        for value in values {
            append_query(out, key, value);
        }
    }
}

pub fn markets_query(params: &crate::public_client::ListMarketsRequest) -> Vec<(String, String)> {
    let mut query = Vec::new();
    append_bool(&mut query, "closed", params.closed);
    append_optional(&mut query, "limit", params.page_size.as_ref());
    if let Some(cursor) = &params.cursor {
        append_query(&mut query, "after_cursor", cursor);
    }
    append_optional(&mut query, "order", params.order.as_deref());
    append_bool(&mut query, "ascending", params.ascending);
    append_string_slice(&mut query, "slug", params.slug.as_deref());
    append_optional(&mut query, "tag_id", params.tag_id.as_ref());
    query
}

pub fn events_query(params: &crate::public_client::ListEventsRequest) -> Vec<(String, String)> {
    let mut query = Vec::new();
    append_bool(&mut query, "closed", params.closed);
    append_optional(&mut query, "limit", params.page_size.as_ref());
    if let Some(cursor) = &params.cursor {
        append_query(&mut query, "after_cursor", cursor);
    }
    append_optional(&mut query, "order", params.order.as_deref());
    append_bool(&mut query, "ascending", params.ascending);
    append_bool(&mut query, "featured", params.featured);
    query
}

pub fn as_reqwest_pairs(query: &[(String, String)]) -> Vec<(&str, String)> {
    query.iter().map(|(k, v)| (k.as_str(), v.clone())).collect()
}
