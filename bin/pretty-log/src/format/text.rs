use std::io::{self, Write};

use super::{InputFormat, OutputFormat};
use crate::message::{Message, Severity};

pub struct TextFormat {}

impl TextFormat {
    pub fn new() -> TextFormat {
        TextFormat {}
    }
}

impl InputFormat for TextFormat {
    fn parse_message(&self, message: &str, default_severity: Severity) -> Option<Message> {
        Some(Message::from_text(message, default_severity))
    }
}

impl OutputFormat for TextFormat {
    fn print_message<W: Write + ?Sized>(
        &self,
        writer: &mut W,
        message: &Message,
    ) -> io::Result<()> {
        writeln!(writer, "{}", message.text)
    }
}
