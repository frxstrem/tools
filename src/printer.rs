use std::marker::PhantomData;
use std::sync::{Mutex, MutexGuard};

use crate::args::Args;
use crate::ext::*;
use crate::message::{Message, Severity};

pub trait MessagePrinter: Send + Sync {
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
    lock: Mutex<()>,
    _styling: PhantomData<S>,
}

impl<S: Styling> FancyPrinter<S> {
    pub fn new(args: &Args) -> FancyPrinter<S> {
        let args = args.clone();
        FancyPrinter {
            args,
            lock: Mutex::new(()),
            _styling: PhantomData,
        }
    }

    fn lock(&self) -> MutexGuard<'_, ()> {
        self.lock.lock().unwrap()
    }
}

impl<S: Styling> MessagePrinter for FancyPrinter<S> {
    fn print(&self, message: &Message) {
        let _lock = self.lock();

        let lines: Vec<_> = message
            .to_string()
            .split('\n')
            .map(str::to_string)
            .collect();

        let mut extras = Vec::new();

        if self.args.show_context {
            let context = message.context();
            if !context.is_empty() {
                extras.push(format!("{:?}", context));
            }
        } else if self.args.show_request_id {
            if let Some(request_id) = message.request_id() {
                extras.push(format!("[{}]", request_id));
            }
        }

        if self.args.show_source {
            if let Some(source_location) = message.source_location().and_then(|loc| loc.to_string())
            {
                extras.push(format!("({})", source_location));
            }
        }

        if self.args.debug {
            extras.push(format!("{:?}", message));
        }

        for (lineno, line) in lines.into_iter().enumerate() {
            let is_first = lineno == 0;

            // print prefix fields
            print!("{}", S::severity(message.severity()));
            print!("{}", S::weak());

            if let Some(timestamp) = message.timestamp().and_if(|| is_first) {
                print!("{}", timestamp.format("%Y-%m-%dT%H:%M:%S%.3f%:z"));
            } else {
                print!("{:29}", "");
            }

            if is_first {
                print!(" {:>9}", message.severity().to_string().to_uppercase());
            } else {
                print!(" {:>9}", "");
            }

            if is_first {
                print!("> ");
            } else {
                print!("… ");
            }

            if let Some(process_name) = message.process_name().and_if(|| self.args.show_process) {
                if is_first {
                    print!("[{}] ", process_name);
                } else {
                    print!("{}", " ".repeat(3 + process_name.len()));
                }
            }

            print!("{}", S::reset());
            print!("{}", S::severity(message.severity()));
            print!("{}", S::strong());
            print!("{}", line);

            if is_first && self.args.compact {
                print!("{}", S::reset());
                print!("{}", S::severity(message.severity()));
                print!("{}", S::weak());

                for extra in extras.iter() {
                    print!(" {}", extra);
                }
            }

            println!("{}", S::reset());
        }

        if !self.args.compact {
            for extra in extras.into_iter() {
                print!("{}", S::severity(message.severity()));
                print!("{}", S::weak());

                print!("{:39}+ {}", "", extra);

                println!("{}", S::reset());
            }
        }
    }
}

pub struct ColorStyling;
pub struct NoColorStyling;

pub trait Styling: Send + Sync + private::Sealed {
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
