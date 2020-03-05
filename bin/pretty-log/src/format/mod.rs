mod go;
mod json;
mod pretty;
pub mod style;
mod text;

use std::io::{self, Write};

use self::style::Style;
use crate::message::{Message, Severity};
use crate::DisplayOptions;

macro_rules! format_select {
    (
        select_fn = $select_fn:ident
            $( (
                    $($arg:ident : $arg_type:ty),* $(,)?
            ) )?
        ;
        variants_fn = $variants_fn:ident;
        default_fn = $default_fn:ident;
        type = $type:ty;
        default = $default:literal;

        $(
            $($pattern:literal)|+ => $expr:expr
        ),*
        $(,)?
    ) => {
        pub fn $select_fn(format: &str $(, $($arg:$arg_type),* )?) -> Result<Box<$type>, String> {
            let format: Box<$type> = match format {
                $(
                    $($pattern)|* => Box::new($expr),
                )*
                _ => return Err(format!("Unknown format: {}", format)),
            };
            Ok(format)
        }

        #[allow(dead_code)]
        pub fn $variants_fn() -> &'static [&'static str] {
            &[ $( $($pattern),* ),* ]
        }

        #[allow(dead_code)]
        pub fn $default_fn() -> &'static str {
            $default
        }
    };
}

format_select! {
    select_fn = get_input_format_impl;
    variants_fn = get_input_format_variants;
    default_fn = get_input_format_default;
    type = dyn InputFormat;
    default = "json,go";

    "json" => json::JsonFormat::new(text::TextFormat::new()),
    "text" => text::TextFormat::new(),
    "go" => go::GoFormat::new(text::TextFormat::new()),
}

format_select! {
    select_fn = get_output_format(style: impl Style, display_opts: &DisplayOptions);
    variants_fn = get_output_format_variants;
    default_fn = get_output_format_default;
    type = dyn DynOutputFormat;
    default = "pretty";

    "text" => text::TextFormat::new(),
    "pretty" => pretty::PrettyFormat::new(style, display_opts),
}

pub fn get_input_format(formats: &[impl AsRef<str>]) -> Result<Box<dyn InputFormat>, String> {
    if formats.is_empty() {
        Ok(Box::new(text::TextFormat::new()))
    } else if formats.len() == 1 {
        get_input_format_impl(formats[0].as_ref())
    } else {
        let formats = formats
            .iter()
            .map(|format| get_input_format_impl(format.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Box::new(ListInputFormat(formats)))
    }
}

pub trait InputFormat: Send + Sync {
    fn parse_message(&self, message: &str, default_severity: Severity) -> Option<Message>;
}

impl<T: InputFormat + ?Sized> InputFormat for &'_ T {
    fn parse_message(&self, message: &str, default_severity: Severity) -> Option<Message> {
        T::parse_message(self, message, default_severity)
    }
}

impl<T: InputFormat + ?Sized> InputFormat for Box<T> {
    fn parse_message(&self, message: &str, default_severity: Severity) -> Option<Message> {
        T::parse_message(self, message, default_severity)
    }
}

pub struct ListInputFormat(Vec<Box<dyn InputFormat>>);

impl InputFormat for ListInputFormat {
    fn parse_message(&self, message: &str, default_severity: Severity) -> Option<Message> {
        for format in &self.0 {
            if let Some(message) = format.parse_message(message, default_severity) {
                return Some(message);
            }
        }

        None
    }
}

pub trait OutputFormat: Send + Sync {
    fn print_message<W: Write + ?Sized>(&self, writer: &mut W, message: &Message)
        -> io::Result<()>;
}

impl<T: OutputFormat + ?Sized> OutputFormat for &'_ T {
    fn print_message<W: Write + ?Sized>(
        &self,
        writer: &mut W,
        message: &Message,
    ) -> io::Result<()> {
        T::print_message(self, writer, message)
    }
}

impl<T: OutputFormat + ?Sized> OutputFormat for Box<T> {
    fn print_message<W: Write + ?Sized>(
        &self,
        writer: &mut W,
        message: &Message,
    ) -> io::Result<()> {
        T::print_message(self, writer, message)
    }
}

pub trait DynOutputFormat: Send + Sync {
    fn print_message_dyn(&self, writer: &mut dyn Write, message: &Message) -> io::Result<()>;
}

impl<T: OutputFormat + ?Sized> DynOutputFormat for T {
    fn print_message_dyn(&self, writer: &mut dyn Write, message: &Message) -> io::Result<()> {
        T::print_message(self, writer, message)
    }
}

impl OutputFormat for dyn DynOutputFormat {
    fn print_message<W: Write + ?Sized>(
        &self,
        mut writer: &mut W,
        message: &Message,
    ) -> io::Result<()> {
        self.print_message_dyn(&mut writer, message)
    }
}
