use std::io::{self, BufRead, BufReader};

mod args;
use args::Args;

mod message;
use message::Message;

mod printer;
use printer::MessagePrinter;

fn main() {
    let args = Args::parse();

    let printer = printer::PlainPrinter;

    // read line by line from standard input
    let reader = BufReader::new(io::stdin());
    for line in reader.lines().map(Result::unwrap) {
        // try to parse line as JSON, or create standard message from raw line
        let message: Message =
            serde_json::from_str(&line).unwrap_or_else(|_| Message::from_raw(line));

        printer.print(&message);
    }
}
