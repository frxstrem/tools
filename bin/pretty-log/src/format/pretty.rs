use std::io::{self, Write};

use super::{style::*, OutputFormat};
use crate::ext::*;
use crate::message::Message;

pub struct PrettyFormat {
    style: AnyStyle,
}

impl PrettyFormat {
    pub fn new<S: Style>(style: S) -> PrettyFormat {
        PrettyFormat {
            style: style.into(),
        }
    }
}

impl OutputFormat for PrettyFormat {
    fn print_message<W: Write + ?Sized>(
        &self,
        writer: &mut W,
        message: &Message,
    ) -> io::Result<()> {
        let lines: Vec<_> = message.text.split('\n').map(str::to_string).collect();

        for (lineno, line) in lines.into_iter().enumerate() {
            let is_first = lineno == 0;

            // print prefix fields
            self.style.severity(writer, message.severity)?;
            self.style.weak(writer)?;

            if let Some(timestamp) = message.time.as_ref().and_if(|| is_first) {
                print!("{}", timestamp.format("%Y-%m-%dT%H:%M:%S%.3f%:z"));
            } else {
                print!("{:29}", "");
            }

            if is_first {
                print!(" {:>9}", message.severity.to_string().to_uppercase());
            } else {
                print!(" {:>9}", "");
            }

            if is_first {
                print!("> ");
            } else {
                print!("… ");
            }

            self.style.reset(writer)?;
            self.style.severity(writer, message.severity)?;
            self.style.strong(writer)?;
            write!(writer, "{}", line)?;

            self.style.reset(writer)?;

            writeln!(writer)?;
        }

        Ok(())
    }
}
