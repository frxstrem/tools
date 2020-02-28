use std::io::{self, Write};

use crate::message::Severity;

pub trait Style: Into<AnyStyle> {
    fn reset<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()>;
    fn severity<W: Write + ?Sized>(&self, writer: &mut W, severity: Severity) -> io::Result<()>;
    fn weak<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()>;
    fn strong<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()>;
}

pub enum AnyStyle {
    Plain(PlainStyle),
    Colored(ColoredStyle),
}

impl Style for AnyStyle {
    fn reset<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            AnyStyle::Plain(style) => style.reset(writer),
            AnyStyle::Colored(style) => style.reset(writer),
        }
    }

    fn severity<W: Write + ?Sized>(&self, writer: &mut W, severity: Severity) -> io::Result<()> {
        match self {
            AnyStyle::Plain(style) => style.severity(writer, severity),
            AnyStyle::Colored(style) => style.severity(writer, severity),
        }
    }

    fn weak<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            AnyStyle::Plain(style) => style.weak(writer),
            AnyStyle::Colored(style) => style.weak(writer),
        }
    }

    fn strong<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            AnyStyle::Plain(style) => style.strong(writer),
            AnyStyle::Colored(style) => style.strong(writer),
        }
    }
}

pub struct PlainStyle;

impl From<PlainStyle> for AnyStyle {
    fn from(style: PlainStyle) -> AnyStyle {
        AnyStyle::Plain(style)
    }
}

impl Style for PlainStyle {
    fn reset<W: Write + ?Sized>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn severity<W: Write + ?Sized>(&self, _writer: &mut W, _severity: Severity) -> io::Result<()> {
        Ok(())
    }

    fn weak<W: Write + ?Sized>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn strong<W: Write + ?Sized>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

pub struct ColoredStyle;

impl From<ColoredStyle> for AnyStyle {
    fn from(style: ColoredStyle) -> AnyStyle {
        AnyStyle::Colored(style)
    }
}

impl Style for ColoredStyle {
    fn reset<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        write!(writer, "\x1b[0m")
    }

    fn severity<W: Write + ?Sized>(&self, writer: &mut W, severity: Severity) -> io::Result<()> {
        if severity >= Severity::Error {
            write!(writer, "\x1b[31m")
        } else if severity >= Severity::Warning {
            write!(writer, "\x1b[33m")
        } else if severity >= Severity::Info {
            write!(writer, "\x1b[34m")
        } else if severity >= Severity::Debug {
            write!(writer, "\x1b[32m")
        } else {
            Ok(())
        }
    }

    fn weak<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        write!(writer, "\x1b[2m")
    }

    fn strong<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        write!(writer, "\x1b[1m")
    }
}
