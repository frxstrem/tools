use std::marker::PhantomData;
use std::sync::{Mutex, MutexGuard};

use super::styling::*;
use super::MessagePrinter;
use crate::args::Args;
use crate::ext::*;
use crate::message::Message;

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
                const PROCESS_NAME_MIN_WIDTH: usize = 8;
                if is_first {
                    if process_name.len() < PROCESS_NAME_MIN_WIDTH {
                        print!(
                            "[{:}] {}",
                            process_name,
                            " ".repeat(PROCESS_NAME_MIN_WIDTH - process_name.len())
                        );
                    } else {
                        print!("[{:}] ", process_name);
                    }
                } else {
                    print!(
                        "{:13}",
                        " ".repeat(3 + process_name.len().max(PROCESS_NAME_MIN_WIDTH))
                    );
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

    fn emphasize(&self, text: &str) -> String {
        format!("{}{}{}", S::emphasize(), text, S::no_emphasize())
    }
}
