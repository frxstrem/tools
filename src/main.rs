use crossbeam::scope;
use std::error::Error;
use std::io::{self, BufRead, BufReader, Read};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;

mod args;
use args::{Args, SeverityRange};

#[macro_use]
mod macros;

mod ext;

mod message;
use message::{Message, Severity};

mod printer;
use printer::MessagePrinter;

use regex::{Captures, Regex};

fn main() {
    match app_main() {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(err) => {
            println!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn app_main() -> Result<i32, Box<dyn Error>> {
    let args = Args::parse();

    let printer = get_printer(&args);

    let grep = args
        .grep
        .as_ref()
        .map(|pattern| Regex::new(pattern).unwrap())
        .map(|regex| (regex, args.grep_show_all));

    match args.command {
        Some(cmd) => {
            let command = &cmd[0];
            let cmd_args = &cmd[1..];

            let exit_status = run_command(
                printer.into(),
                &command,
                cmd_args,
                args.severity_range,
                grep,
                args.request_id.as_ref().map(String::as_str),
                args.raw,
            )?;
            Ok(exit_status.code().unwrap_or(255))
        }
        None => {
            // read line by line from standard input
            printer_loop(
                io::stdin(),
                printer.as_ref(),
                Severity::Default,
                args.severity_range,
                grep.as_ref(),
                args.request_id.as_ref().map(String::as_str),
                args.raw,
            );
            Ok(0)
        }
    }
}

fn get_printer(args: &Args) -> Box<dyn MessagePrinter> {
    let plain = args.plain;
    let colored = args.colored.unwrap_or_else(auto_color);

    match (plain, colored) {
        (true, true) => Box::new(printer::PlainPrinter::<printer::ColorStyling>::default()),
        (true, false) => Box::new(printer::PlainPrinter::<printer::NoColorStyling>::default()),
        (false, true) => Box::new(printer::FancyPrinter::<printer::ColorStyling>::new(args)),
        (false, false) => Box::new(printer::FancyPrinter::<printer::NoColorStyling>::new(args)),
    }
}

fn auto_color() -> bool {
    unsafe { libc_result!(libc::isatty(1)).unwrap() > 0 }
}

fn printer_loop<R: Read>(
    reader: R,
    printer: &dyn MessagePrinter,
    default_severity: Severity,
    severity_range: SeverityRange,
    grep: Option<&(Regex, bool)>,
    request_id: Option<&str>,
    raw: bool,
) {
    let reader = BufReader::new(reader);
    for line in reader.lines().map(Result::unwrap) {
        // try to parse line as JSON, or create standard message from raw line
        let mut message: Message = if raw {
            Message::from_raw(line)
        } else {
            serde_json::from_str(&line).unwrap_or_else(|_| Message::from_raw(line))
        };

        if let Some(request_id) = request_id {
            match message.request_id() {
                Some(message_request_id) => {
                    if message_request_id != request_id {
                        continue;
                    }
                }
                None => continue,
            }
        }

        if let Some((grep, show_all)) = grep.as_ref() {
            if !show_all && !grep.is_match(message.text()) {
                continue;
            }

            let text = grep
                .replace_all(message.text(), |captures: &Captures| {
                    printer.emphasize(captures.get(0).unwrap().as_str())
                })
                .into_owned();
            message.set_text(&text);
        }

        message.set_default_severity(default_severity);

        if let Some(min_severity) = severity_range.0 {
            if message.severity() < min_severity {
                continue;
            }
        }
        if let Some(max_severity) = severity_range.1 {
            if message.severity() > max_severity {
                continue;
            }
        }

        printer.print(&message);
    }
}

fn run_command(
    printer: Arc<dyn MessagePrinter>,
    command: &str,
    cmd_args: &[String],
    severity_range: SeverityRange,
    grep: Option<(Regex, bool)>,
    request_id: Option<&str>,
    raw: bool,
) -> Result<ExitStatus, Box<dyn Error>> {
    let mut child = Command::new(command)
        .args(cmd_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("take stdout");
    let stderr = child.stderr.take().expect("take stderr");

    scope(|s| {
        s.spawn(|_| {
            printer_loop(
                stdout,
                printer.as_ref(),
                Severity::Default,
                severity_range,
                grep.as_ref(),
                request_id,
                raw,
            )
        });

        s.spawn(|_| {
            printer_loop(
                stderr,
                printer.as_ref(),
                Severity::Error,
                severity_range,
                grep.as_ref(),
                request_id,
                raw,
            )
        });

        child.wait()
    })
    .unwrap()
    .map_err(|err| err.into())
}
