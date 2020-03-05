use chrono::{DateTime, Local};
use serde::{de::Deserializer, Deserialize};
use std::collections::HashMap;
use std::fmt::{self, Display};

use crate::utils::StringOrNumber;

#[derive(Clone, Default, Debug)]
pub struct Message {
    pub text: String,
    pub severity: Severity,
    pub time: Option<DateTime<Local>>,
    pub context: HashMap<String, String>,
}

impl Message {
    pub fn from_text(message: impl AsRef<str>, severity: Severity) -> Message {
        Message {
            text: message.as_ref().to_string(),
            severity,
            time: None,
            context: HashMap::new(),
        }
    }

    pub fn merge_with(&mut self, message: Message) {
        self.text = message.text;
        self.severity = message.severity.or(self.severity);
        self.time = message.time.or(self.time);
    }

    pub fn add_context(&mut self, name: impl AsRef<str>, value: impl AsRef<str>) {
        self.context
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
    }

    pub fn trim(mut self) -> Message {
        self.text = self.text.trim_end_matches('\n').to_string();
        self
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Severity {
    Default,
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
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

impl Severity {
    pub fn or(self, other: Severity) -> Severity {
        if self != Severity::Default {
            self
        } else {
            other
        }
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
