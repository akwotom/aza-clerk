/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This data structure represents a full amount in the system.
*/

/// This data structure represents a complete amount on the system.
#[derive(Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, std::fmt::Debug)]
pub struct Amount {
    #[sqlx(rename = "amount_value")]
    pub value: i32,
    #[sqlx(rename = "amount_currency")]
    #[serde(deserialize_with = "uppercase")]
    pub currency: String,
}

fn uppercase<'t, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'t>,
{
    Result::Ok(<String as serde::Deserialize>::deserialize(deserializer)?.to_uppercase())
}
