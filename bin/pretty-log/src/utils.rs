use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum StringOrNumber<N> {
    String(String),
    Number(N),
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
