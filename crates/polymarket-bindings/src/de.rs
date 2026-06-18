use serde::{Deserialize, Deserializer};

/// Deserialize a decimal value that may arrive as a string or number from Gamma/CLOB APIs.
pub fn deserialize_decimalish<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<DecimalishValue>::deserialize(deserializer)?;
    Ok(value.map(DecimalishValue::into_string))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DecimalishValue {
    String(String),
    Float(f64),
    Int(i64),
    UInt(u64),
}

impl DecimalishValue {
    fn into_string(self) -> String {
        match self {
            Self::String(s) => s,
            Self::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{f:.0}")
                } else {
                    f.to_string()
                }
            }
            Self::Int(i) => i.to_string(),
            Self::UInt(u) => u.to_string(),
        }
    }
}

/// Deserialize JSON string arrays or inline arrays.
pub fn deserialize_string_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(|v| match v {
                serde_json::Value::String(s) => Ok(s),
                serde_json::Value::Number(n) => Ok(n.to_string()),
                other => Err(serde::de::Error::custom(format!(
                    "expected string array element, got {other}"
                ))),
            })
            .collect(),
        serde_json::Value::String(json) => {
            serde_json::from_str(&json).map_err(serde::de::Error::custom)
        }
        serde_json::Value::Null => Ok(Vec::new()),
        other => Err(serde::de::Error::custom(format!(
            "expected string or array, got {other}"
        ))),
    }
}

pub fn deserialize_empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.filter(|s| !s.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Sample {
        #[serde(deserialize_with = "deserialize_decimalish")]
        value: Option<String>,
    }

    #[test]
    fn parses_float_decimal() {
        let s: Sample = serde_json::from_str(r#"{"value": 0.01}"#).unwrap();
        assert_eq!(s.value.as_deref(), Some("0.01"));
    }

    #[test]
    fn parses_string_decimal() {
        let s: Sample = serde_json::from_str(r#"{"value": "0.52"}"#).unwrap();
        assert_eq!(s.value.as_deref(), Some("0.52"));
    }
}
