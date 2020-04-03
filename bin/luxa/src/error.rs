use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub enum LuxaError {
    Hid(hidapi::HidError),
    Other(String),
}

impl LuxaError {
    pub fn new<E: ToString>(err: E) -> LuxaError {
        LuxaError::Other(err.to_string())
    }
}

impl Display for LuxaError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            LuxaError::Hid(err) => Display::fmt(err, fmt),
            LuxaError::Other(message) => write!(fmt, "{}", message),
        }
    }
}

impl Error for LuxaError {}

impl From<hidapi::HidError> for LuxaError {
    fn from(err: hidapi::HidError) -> LuxaError {
        LuxaError::Hid(err)
    }
}
