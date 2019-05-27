use std::marker::PhantomData;

use crate::args::Args;
use crate::ext::*;
use crate::message::{Message, Severity};

pub trait MessagePrinter {
    fn print(&self, message: &Message);
}

pub struct PlainPrinter;

impl MessagePrinter for PlainPrinter {
    fn print(&self, message: &Message) {
        println!("{}", message);
    }
}

pub struct FancyPrinter<S: Styling> {
    args: Args,
    _styling: PhantomData<S>,
}

impl<S: Styling> FancyPrinter<S> {
    pub fn new(args: &Args) -> FancyPrinter<S> {
        let args = args.clone();
        FancyPrinter {
            args,
            _styling: PhantomData,
        }
    }
}

impl<S: Styling> MessagePrinter for FancyPrinter<S> {
    fn print(&self, message: &Message) {
        let mut lines: Vec<_> = message
            .to_string()
            .split('\n')
            .map(str::to_string)
            .map(|line| (line, true))
            .collect();

        let mut append = |line| lines.push((line, false));

        if self.args.show_context {
            let context = message.context();
            if !context.is_empty() {
                append(format!("ctx = {:?}", context));
            }
        }

        if self.args.show_source {
            if let Some(source_location) = message.source_location().and_then(|loc| loc.to_string())
            {
                append(format!("({})", source_location));
            }
        }

        if self.args.debug {
            append(format!("{:?}", message));
        }

        for (lineno, (line, strong)) in lines.into_iter().enumerate() {
            let is_first = lineno == 0;

            // print prefix fields
            print!("{}", S::severity(message.severity()));
            print!("{}", S::weak());

            if let Some(timestamp) = message.timestamp().and_if(|| is_first) {
                print!("{}", timestamp.format("%Y-%m-%dT%H:%M:%S%.3f%:z"));
            } else {
                print!("{:29}", "");
            }

            if let Some(request_id) = message.request_id().and_if(|| self.args.show_request_id) {
                if is_first {
                    print!(" [{}]", request_id);
                } else {
                    print!("{}", " ".repeat(3 + request_id.len()));
                }
            }

            if is_first {
                print!(" {:>9}", message.severity().to_string().to_uppercase());
            } else {
                print!(" {:>9}", "");
            }

            if is_first {
                print!("> ");
            } else if strong {
                print!("… ");
            } else {
                print!("+ ");
            }

            if let Some(process_name) = message
                .process_name()
                .and_if(|| self.args.show_process)
            {
                if is_first {
                    print!("[{}] ", process_name);
                } else {
                    print!("{}", " ".repeat(3 + process_name.len()));
                }
            }

            if strong {
                print!("{}", S::reset());
                print!("{}", S::severity(message.severity()));
                print!("{}", S::strong());
            }

            print!("{}", line);

            print!("{}", S::reset());
            println!();
        }
    }
}

pub struct ColorStyling;
pub struct NoColorStyling;

pub trait Styling: private::Sealed {
    fn severity(_severity: Severity) -> &'static str {
        ""
    }
    fn weak() -> &'static str {
        ""
    }
    fn strong() -> &'static str {
        ""
    }
    fn reset() -> &'static str {
        ""
    }
}

impl Styling for ColorStyling {
    fn severity(severity: Severity) -> &'static str {
        if severity >= Severity::Error {
            "\x1b[31m"
        } else if severity >= Severity::Warning {
            "\x1b[33m"
        } else if severity >= Severity::Info {
            "\x1b[34m"
        } else if severity >= Severity::Debug {
            "\x1b[32m"
        } else {
            ""
        }
    }

    fn weak() -> &'static str {
        "\x1b[2m"
    }
    fn strong() -> &'static str {
        "\x1b[1m"
    }
    fn reset() -> &'static str {
        "\x1b[0m"
    }
}

impl Styling for NoColorStyling {}

mod private {
    pub trait Sealed {}
    impl Sealed for super::ColorStyling {}
    impl Sealed for super::NoColorStyling {}
}
