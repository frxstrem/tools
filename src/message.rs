use std::collections::HashMap;
use std::fmt::{self, Display};

use chrono::prelude::*;
use serde::{de, Deserialize, Deserializer};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Message {
    /// Text content of log message.
    #[serde(rename = "message")]
    text: String,

    /// Log message severity.
    severity: Severity,

    /// Timestamp of log message (in local time).
    #[serde(alias = "time", default, deserialize_with = "parse_datetime")]
    timestamp: Option<DateTime<Local>>,

    /// Originating location of message in source code.
    #[serde(rename = "sourceLocation", alias = "src")]
    source_location: Option<SourceLocation>,

    /// Name of process that logged this message.
    #[serde(rename = "processName")]
    process_name: Option<String>,

    /// Arbitrary context data
    context: HashMap<String, String>,
}

impl Message {
    pub fn from_raw(text: impl AsRef<str>) -> Message {
        Message {
            text: text.as_ref().to_string(),
            ..Default::default()
        }
    }
}

impl Display for Message {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.text)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Severity {
    Default = 0,
    Debug = 100,
    Info = 200,
    Notice = 300,
    Warning = 400,
    Error = 500,
    Critical = 600,
    Alert = 700,
    Emergency = 800,
}

impl Default for Severity {
    fn default() -> Severity {
        Severity::Default
    }
}

impl<'de> Deserialize<'de> for Severity {
    fn deserialize<D>(deserializer: D) -> Result<Severity, D::Error>
    where
        D: Deserializer<'de>,
    {
        match StringOrNumber::<u64>::deserialize(deserializer)? {
            StringOrNumber::String(severity) => match severity.to_lowercase().as_ref() {
                "emergency" => Ok(Severity::Emergency),
                "alert" => Ok(Severity::Alert),
                "critical" => Ok(Severity::Critical),
                "error" => Ok(Severity::Error),
                "warning" => Ok(Severity::Warning),
                "notice" => Ok(Severity::Notice),
                "info" => Ok(Severity::Info),
                "debug" => Ok(Severity::Debug),
                _ => Ok(Severity::Default),
            },
            StringOrNumber::Number(severity) => {
                if severity >= (Severity::Emergency as u64) {
                    Ok(Severity::Emergency)
                } else if severity >= (Severity::Alert as u64) {
                    Ok(Severity::Alert)
                } else if severity >= (Severity::Critical as u64) {
                    Ok(Severity::Critical)
                } else if severity >= (Severity::Error as u64) {
                    Ok(Severity::Error)
                } else if severity >= (Severity::Warning as u64) {
                    Ok(Severity::Warning)
                } else if severity >= (Severity::Notice as u64) {
                    Ok(Severity::Notice)
                } else if severity >= (Severity::Info as u64) {
                    Ok(Severity::Info)
                } else if severity >= (Severity::Debug as u64) {
                    Ok(Severity::Debug)
                } else {
                    Ok(Severity::Default)
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceLocation {
    file: Option<String>,

    #[serde(default, deserialize_with = "parse_usize_opt")]
    line: Option<usize>,

    #[serde(default, deserialize_with = "parse_usize_opt")]
    column: Option<usize>,

    function: Option<String>,
}

fn parse_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TimestampVariants {
        DateTime(DateTime<Local>),
        SecondsNanos { seconds: i64, nanos: u32 },
    }

    match <Option<TimestampVariants> as Deserialize>::deserialize(deserializer)? {
        Some(TimestampVariants::DateTime(datetime)) => Ok(Some(datetime)),
        Some(TimestampVariants::SecondsNanos { seconds, nanos }) => {
            Ok(Some(Local.timestamp(seconds, nanos)))
        }
        None => Ok(None),
    }
}

fn parse_usize_opt<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    match <Option<StringOrNumber<_>> as Deserialize>::deserialize(deserializer)? {
        Some(StringOrNumber::String(value)) => {
            Ok(Some(value.parse().map_err(|err| de::Error::custom(err))?))
        }
        Some(StringOrNumber::Number(value)) => Ok(value),
        None => Ok(None),
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrNumber<N> {
    String(String),
    Number(N),
}
