use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StringOrNumber<N> {
    String(String),
    Number(N),
}

impl<N: FromStr> StringOrNumber<N> {
    pub fn into_number(self) -> Result<N, N::Err> {
        match self {
            StringOrNumber::String(s) => s.parse(),
            StringOrNumber::Number(n) => Ok(n),
        }
    }
}

pub fn is_stdout_tty() -> bool {
    unsafe {
        let result = {
            let result = libc::isatty(1);
            if result < 0 {
                ::std::result::Result::Err(::std::io::Error::last_os_error())
            } else {
                ::std::result::Result::Ok(result)
            }
        };

        result.unwrap() > 0
    }
}
