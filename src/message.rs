use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Deserialize)]
pub struct Message {
    text: String,
    severity: Severity,
}

impl Message {
    pub fn from_raw(text: impl AsRef<str>) -> Message {
        Message {
            text: text.as_ref().to_string(),
            severity: Severity::Default,
        }
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
        #[derive(Deserialize)]
        enum StringOrNumber {
            String(String),
            Number(u64),
        }

        match StringOrNumber::deserialize(deserializer)? {
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
