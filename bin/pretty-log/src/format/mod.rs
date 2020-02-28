mod json;
mod pretty;
pub mod style;
mod text;

use std::io::{self, Write};

use self::style::Style;
use crate::message::{Message, Severity};

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
        pub fn $select_fn(format: &str $(, $($arg:$arg_type),* )?) -> Option<Box<$type>> {
            let format: Box<$type> = match format {
                $(
                    $($pattern)|* => Box::new($expr),
                )*
                _ => return None,
            };
            Some(format)
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
    select_fn = get_input_format;
    variants_fn = get_input_format_variants;
    default_fn = get_input_format_default;
    type = dyn InputFormat;
    default = "json";

    "json" => json::JsonFormat::new(text::TextFormat::new()),
    "text" => text::TextFormat::new(),
}

format_select! {
    select_fn = get_output_format(style: impl Style);
    variants_fn = get_output_format_variants;
    default_fn = get_output_format_default;
    type = dyn DynOutputFormat;
    default = "pretty";

    "text" => text::TextFormat::new(),
    "pretty" => pretty::PrettyFormat::new(style),
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
