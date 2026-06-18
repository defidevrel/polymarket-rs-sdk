use serde::Deserialize;

use crate::de::deserialize_decimalish;

#[derive(Debug, Deserialize)]
pub struct MidpointResponse {
    #[serde(deserialize_with = "deserialize_decimalish")]
    pub mid: Option<String>,
}

impl MidpointResponse {
    pub fn into_mid(self) -> Result<String, String> {
        self.mid.ok_or_else(|| "missing midpoint".to_string())
    }
}
