use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

pub struct LuxaError(hidapi::HidError);

impl Debug for LuxaError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.0)
    }
}

impl Display for LuxaError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl Error for LuxaError {}

impl From<hidapi::HidError> for LuxaError {
    fn from(err: hidapi::HidError) -> LuxaError {
        LuxaError(err)
    }
}
