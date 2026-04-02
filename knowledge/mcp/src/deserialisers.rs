use rmcp::serde_json;
use serde::Deserialize;

pub fn deserialize_usize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    match serde_json::Value::deserialize(d)? {
        serde_json::Value::Number(n) => n
            .as_u64()
            .map(|n| n as usize)
            .ok_or_else(|| serde::de::Error::custom("expected non-negative integer")),
        serde_json::Value::String(s) => s.parse().map_err(serde::de::Error::custom),
        other => Err(serde::de::Error::custom(format!(
            "expected integer, got {other}"
        ))),
    }
}
