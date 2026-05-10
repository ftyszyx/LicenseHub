use std::str::FromStr;
use std::fmt::Display;
use serde::de;
pub fn from_str<'de, D,T>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s: String = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}


pub fn from_str_optional<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s: Option<String> = Option::deserialize(deserializer)?;
    s.map(|s| T::from_str(&s).map_err(de::Error::custom))
        .transpose()
}