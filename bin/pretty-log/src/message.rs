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
    severity: Option<Severity>,

    /// Timestamp of log message (in local time).
    #[serde(alias = "time", default, deserialize_with = "parse_datetime")]
    timestamp: Option<DateTime<Local>>,

    /// Originating location of message in source code.
    #[serde(rename = "sourceLocation", alias = "src")]
    source_location: Option<SourceLocation>,

    /// Arbitrary context data
    #[serde(default)]
    context: HashMap<String, String>,
}

impl Message {
    pub fn from_raw(text: impl AsRef<str>) -> Message {
        Message {
            text: text.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn set_default_severity(&mut self, default_severity: Severity) {
        if self.severity.is_none() {
            self.severity = Some(default_severity)
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn severity(&self) -> Severity {
        self.severity.unwrap_or(Severity::Default)
    }

    pub fn timestamp(&self) -> Option<DateTime<Local>> {
        self.timestamp
    }

    pub fn source_location(&self) -> Option<&SourceLocation> {
        self.source_location.as_ref()
    }

    pub fn process_name(&self) -> Option<&str> {
        self.context.get("processName").map(String::as_ref)
    }

    pub fn context(&self) -> &HashMap<String, String> {
        &self.context
    }

    pub fn request_id(&self) -> Option<&str> {
        self.context.get("requestId").map(String::as_ref)
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

impl Severity {
    pub fn validate_str(severity: String) -> Result<(), String> {
        Severity::try_parse_str(&severity).map(|_| ())
    }

    pub fn try_parse_str(severity: &str) -> Result<Severity, String> {
        if let Ok(severity) = severity.parse::<u64>() {
            return Ok(Severity::parse_u64(severity));
        }

        let severity = severity.to_lowercase();
        match severity.as_str() {
            "emergency" => Ok(Severity::Emergency),
            "alert" => Ok(Severity::Alert),
            "critical" => Ok(Severity::Critical),
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "notice" => Ok(Severity::Notice),
            "info" => Ok(Severity::Info),
            "debug" => Ok(Severity::Debug),
            "default" => Ok(Severity::Default),
            _ => Err(format!("Unknown severity level: {}", severity)),
        }
    }

    pub fn parse_u64(severity: u64) -> Severity {
        if severity >= (Severity::Emergency as u64) {
            Severity::Emergency
        } else if severity >= (Severity::Alert as u64) {
            Severity::Alert
        } else if severity >= (Severity::Critical as u64) {
            Severity::Critical
        } else if severity >= (Severity::Error as u64) {
            Severity::Error
        } else if severity >= (Severity::Warning as u64) {
            Severity::Warning
        } else if severity >= (Severity::Notice as u64) {
            Severity::Notice
        } else if severity >= (Severity::Info as u64) {
            Severity::Info
        } else if severity >= (Severity::Debug as u64) {
            Severity::Debug
        } else {
            Severity::Default
        }
    }
}

impl Default for Severity {
    fn default() -> Severity {
        Severity::Default
    }
}

impl Display for Severity {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl<'de> Deserialize<'de> for Severity {
    fn deserialize<D>(deserializer: D) -> Result<Severity, D::Error>
    where
        D: Deserializer<'de>,
    {
        match StringOrNumber::<u64>::deserialize(deserializer)? {
            StringOrNumber::String(severity) => {
                Ok(Severity::try_parse_str(&severity).unwrap_or_else(|_| Severity::Default))
            }
            StringOrNumber::Number(severity) => Ok(Severity::parse_u64(severity)),
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

impl SourceLocation {
    pub fn to_string(&self) -> Option<String> {
        match self {
            SourceLocation {
                file: Some(file),
                line: Some(line),
                column: Some(column),
                ..
            } => Some(format!("{}:{}:{}", file, line, column)),
            SourceLocation {
                file: Some(file),
                line: Some(line),
                ..
            } => Some(format!("{}:{}", file, line)),
            SourceLocation {
                function: Some(function),
                ..
            } => Some(format!("#{}", function)),
            _ => None,
        }
    }
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

    match <Option<TimestampVariants> as Deserialize>::deserialize(deserializer) {
        Ok(Some(TimestampVariants::DateTime(datetime))) => Ok(Some(datetime)),
        Ok(Some(TimestampVariants::SecondsNanos { seconds, nanos })) => {
            Ok(Some(Local.timestamp(seconds, nanos)))
        }
        _ => Ok(None),
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