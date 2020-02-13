use std::collections::HashMap;

use chrono::{DateTime, Local};

use crate::message::{Message, Severity};

pub trait MessageParser: Send + Sync {
    fn parse(&self, line: &str) -> Result<Message, Box<dyn std::error::Error>>;
}

pub struct RawParser;

impl MessageParser for RawParser {
    fn parse(&self, line: &str) -> Result<Message, Box<dyn std::error::Error>> {
        Ok(Message::from_raw(line, true))
    }
}

pub struct JsonParser;

impl MessageParser for JsonParser {
    fn parse(&self, line: &str) -> Result<Message, Box<dyn std::error::Error>> {
        serde_json::from_str(line).map_err(|err| err.into())
    }
}

pub struct GolangLogParser;

impl MessageParser for GolangLogParser {
    fn parse(&self, line: &str) -> Result<Message, Box<dyn std::error::Error>> {
        // validate log line
        if !regex!(r#"^\s*([a-z_-]+=("(\\"|[^"])*"|[^"\s]+)\s*)+$"#).is_match(line) {
            return Err("Failed to parse go log line".into());
        }

        // split log line
        let mut entries = HashMap::<String, String>::new();
        for m in regex!(r#"(?P<name>[a-z_-]+)=(?P<value>"(\\"|[^"])*"|[^"\s]+)"#).captures_iter(line)
        {
            let name = m.name("name").unwrap().as_str();
            let value = m.name("value").unwrap().as_str();

            let value = if value.starts_with("\"") {
                let value = &value[1..(value.len() - 1)];
                let value = value.replace("\\\"", "\"");
                value
            } else {
                value.to_string()
            };

            entries.insert(name.to_string(), value);
        }

        let text: String = entries.remove("msg").ok_or_else(|| "Missing msg field")?;

        let severity: Option<Severity> =
            entries
                .remove("level")
                .and_then(|level| match level.as_str() {
                    "trace" => Some(Severity::Debug),
                    "debug" => Some(Severity::Debug),
                    "info" => Some(Severity::Info),
                    "warning" => Some(Severity::Warning),
                    "error" => Some(Severity::Error),
                    "panic" => Some(Severity::Alert),
                    "fatal" => Some(Severity::Critical),
                    _ => None,
                });

        let timestamp: Option<DateTime<Local>> = entries
            .remove("time")
            .map(|time| DateTime::parse_from_rfc3339(&time))
            .transpose()?
            .map(|time| time.into());

        Ok(Message {
            text,
            severity,
            timestamp,
            source_location: None,
            context: entries,
        })
    }
}
