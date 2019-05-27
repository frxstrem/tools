use std::io::{self, BufRead, BufReader};

mod args;
use args::{Args, FormattingMode};

#[macro_use]
mod macros;

mod ext;

mod message;
use message::Message;

mod printer;
use printer::MessagePrinter;

fn main() {
    let args = Args::parse();

    let printer = get_printer(&args);

    // read line by line from standard input
    let reader = BufReader::new(io::stdin());
    for line in reader.lines().map(Result::unwrap) {
        // try to parse line as JSON, or create standard message from raw line
        let message: Message =
            serde_json::from_str(&line).unwrap_or_else(|_| Message::from_raw(line));

        printer.print(&message);
    }
}

fn get_printer(args: &Args) -> Box<dyn MessagePrinter> {
    match args.formatting {
        FormattingMode::Plain => Box::new(printer::PlainPrinter),
        FormattingMode::Colored => {
            Box::new(printer::FancyPrinter::<printer::ColorStyling>::new(args))
        }
        FormattingMode::Uncolored => {
            Box::new(printer::FancyPrinter::<printer::NoColorStyling>::new(args))
        }
        FormattingMode::Auto => {
            let isatty = unsafe { libc_result!(libc::isatty(1)).unwrap() > 0 };

            if isatty {
                Box::new(printer::FancyPrinter::<printer::ColorStyling>::new(args))
            } else {
                Box::new(printer::FancyPrinter::<printer::NoColorStyling>::new(args))
            }
        }
    }
}
