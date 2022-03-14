use chrono::{DateTime, Local};
use serde::de;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

// https://stackoverflow.com/questions/57614558/how-to-use-a-custom-serde-deserializer-for-chrono-timestamps
pub fn deserialize_from_str<'de, D>(deserializer: D) -> std::result::Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s: String = Deserialize::deserialize(deserializer)?;
    s += "+08:00";
    DateTime::<Local>::from_str(&s).map_err(de::Error::custom)
}
