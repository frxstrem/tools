use chrono::{DateTime, Local, TimeZone};
use serde::{
    de::{self, Deserializer},
    Deserialize,
};
use std::collections::HashMap;

use super::{text::TextFormat, InputFormat};
use crate::message::{Message, Severity};
use crate::utils::StringOrNumber;

pub struct JsonFormat<T: ?Sized = TextFormat> {
    inner_format: T,
}

impl<T> JsonFormat<T> {
    pub fn new(inner_format: T) -> JsonFormat<T> {
        JsonFormat { inner_format }
    }
}

impl<T: InputFormat + ?Sized> InputFormat for JsonFormat<T> {
    fn parse_message(&self, message: &str, default_severity: Severity) -> Option<Message> {
        let message: JsonMessage = serde_json::from_str(message).ok()?;

        let mut message = message.to_message(default_severity);
        if let Some(inner_message) = self
            .inner_format
            .parse_message(&message.text, Severity::Default)
        {
            message.merge_with(inner_message);
        }

        Some(message)
    }
}

#[derive(Deserialize)]
struct JsonMessage {
    pub message: String,
    pub severity: Option<Severity>,

    #[serde(alias = "timestamp", default, deserialize_with = "parse_time")]
    pub time: Option<DateTime<Local>>,

    #[serde(default)]
    pub context: HashMap<String, String>,
}

impl JsonMessage {
    pub fn to_message(self, default_severity: Severity) -> Message {
        Message {
            text: self.message,
            severity: self.severity.unwrap_or(default_severity),
            time: self.time,
            context: self.context,

            ..Default::default()
        }
    }
}

fn parse_time<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum TimestampVariants {
        DateTime(DateTime<Local>),
        SecondsNanos {
            seconds: StringOrNumber<i64>,
            nanos: StringOrNumber<u32>,
        },
        Any(serde_json::Value),
    }

    match <Option<TimestampVariants> as Deserialize>::deserialize(deserializer) {
        Ok(Some(TimestampVariants::DateTime(datetime))) => Ok(Some(datetime)),
        Ok(Some(TimestampVariants::SecondsNanos { seconds, nanos })) => Ok(Some(Local.timestamp(
            seconds.into_number().map_err(de::Error::custom)?,
            nanos.into_number().map_err(de::Error::custom)?,
        ))),
        _ => Ok(None),
    }
}
