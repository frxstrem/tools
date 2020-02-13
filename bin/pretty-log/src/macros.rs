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

macro_rules! regex {
    ($pattern:literal) => {{
        use ::lazy_static::lazy_static;
        use ::regex::Regex;

        lazy_static! {
            static ref RE: Regex = Regex::new($pattern).unwrap();
        }

        &RE as &Regex
    }};
}
