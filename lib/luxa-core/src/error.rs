use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum LuxaError {
    Hid(LuxaHidError),
}

impl Display for LuxaError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            LuxaError::Hid(LuxaHidError::Message(err)) => write!(fmt, "{}", err),

            #[cfg(feature = "hid")]
            LuxaError::Hid(LuxaHidError::Raw(err)) => write!(fmt, "{}", err),
        }
    }
}

impl Error for LuxaError {}


#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LuxaHidError {
    #[doc(hidden)]
    Message(String),

    #[cfg(feature = "hid")]
    #[doc(hidden)]
    #[serde(serialize_with = "shared::serde::string::serialize", skip_deserializing)]
    Raw(hidapi::HidError),
}

impl ToString for LuxaHidError {
    fn to_string(&self) -> String {
        match self {
            LuxaHidError::Message(err) => err.clone(),

            #[cfg(feature = "hid")]
            LuxaHidError::Raw(err) => err.to_string(),
        }
    }
}

#[cfg(feature = "hid")]
impl From<hidapi::HidError> for LuxaError {
    fn from(err: hidapi::HidError) -> LuxaError {
        LuxaError::Hid(LuxaHidError::Raw(err))
    }
}
