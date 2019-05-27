macro_rules! libc_result {
    ($expr:expr) => {{
        let result = $expr;
        if result < 0 {
            ::std::result::Result::Err(::std::io::Error::last_os_error())
        } else {
            ::std::result::Result::Ok(result)
        }
    }};
}
