use chrono::DateTime;

use super::{text::TextFormat, InputFormat};
use crate::message::{Message, Severity};
use crate::parse::{self, parse, Parse, ParseBuffer, ParseError, Punctuated, Token};

pub struct GoFormat<T: ?Sized = TextFormat> {
    inner_format: T,
}

impl<T> GoFormat<T> {
    pub fn new(inner_format: T) -> GoFormat<T> {
        GoFormat { inner_format }
    }
}

impl<T: InputFormat + ?Sized> InputFormat for GoFormat<T> {
    fn parse_message(&self, message: &str, default_severity: Severity) -> Option<Message> {
        let fields: Punctuated<(RawLiteral, Equals, Value), Whitespace> = parse(message).ok()?;

        let mut message: Message = Message::from_text("", default_severity);

        for (name, _, value) in fields.items() {
            let name: &str = name.as_ref();
            let value: &str = value.as_ref();

            match name {
                "time" => match DateTime::parse_from_rfc3339(value) {
                    Ok(time) => message.time = Some(time.into()),
                    Err(_) => message.add_context(name, value),
                },

                "msg" => message.text = value.to_string(),

                "level" => match parse_severity(value) {
                    Some(severity) => message.severity = severity,
                    None => message.add_context(name, value),
                },

                _ => message.add_context(name, value),
            }
        }

        if let Some(inner_message) = self
            .inner_format
            .parse_message(&message.text, Severity::Default)
        {
            message.merge_with(inner_message);
        }

        Some(message)
    }
}

fn parse_severity(value: &str) -> Option<Severity> {
    match value {
        "debug" => Some(Severity::Debug),
        "info" => Some(Severity::Info),
        "warning" => Some(Severity::Warning),
        "error" => Some(Severity::Error),

        _ => None,
    }
}

macro_rules! regex_token {
    ($vis:vis struct $name:ident = $pattern:literal) => {
        #[derive(Debug)]
        $vis struct $name<'a>(&'a str);

        impl<'a> AsRef<str> for $name<'a> {
            fn as_ref(&self) -> &str {
                self.0
            }
        }

        impl<'a> Token<'a> for $name<'a> {
            fn parse_token(s: &'a str) -> parse::Result<(Self, &'a str)> {
                use lazy_static::lazy_static;
                use regex::Regex;

                lazy_static!{
                    static ref RE: Regex = Regex::new($pattern).unwrap();
                }

                RE.find(s)
                    .and_then(|m| {
                        if m.start() == 0 && m.end() > 0 {
                            Some((Self(m.as_str()), &s[m.end()..]))
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| ParseError::custom(format!("Could not match token {}", stringify!($name))))
            }
        }
    }
}

regex_token!(struct Whitespace = r"[ ]+");
regex_token!(struct RawLiteral = r"[/a-zA-Z0-9_-]+");
regex_token!(struct Equals = r"=");

#[derive(Debug)]
struct StringLiteral(String);

impl AsRef<str> for StringLiteral {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> Token<'a> for StringLiteral {
    fn parse_token(s: &'a str) -> parse::Result<(StringLiteral, &'a str)> {
        let mut iter = s.chars();

        match iter.next() {
            Some('"') => (),
            _ => return Err(ParseError::custom("Invalid string literal")),
        }

        let mut s = String::new();
        while let Some(ch) = iter.next() {
            match ch {
                '"' => return Ok((StringLiteral(s), iter.as_str())),
                '\\' => match iter.next() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some(ch) => {
                        eprintln!("unknown escape sequence: \\{}", ch);
                        s.push('\\');
                        s.push(ch);
                    }
                    None => return Err(ParseError::custom("Invalid string literal")),
                },
                _ => s.push(ch),
            }
        }

        Err(ParseError::custom("Invalid string literal"))
    }
}

#[derive(Debug)]
enum Value<'a> {
    String(StringLiteral),
    Raw(RawLiteral<'a>),
}

impl<'a> AsRef<str> for Value<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Value::String(lit) => lit.as_ref(),
            Value::Raw(lit) => lit.as_ref(),
        }
    }
}

impl<'a> Parse<'a> for Value<'a> {
    fn parse(buf: &mut ParseBuffer<'a>) -> parse::Result<Self> {
        if buf.is_next::<StringLiteral>() {
            StringLiteral::parse(buf).map(Value::String)
        } else if buf.is_next::<RawLiteral>() {
            RawLiteral::parse(buf).map(Value::Raw)
        } else {
            Err(ParseError::custom("Invalid value"))
        }
    }
}
