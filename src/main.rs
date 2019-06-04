use std::error::Error;
use std::io::{self, BufRead, BufReader, Read};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::thread;

mod args;
use args::Args;

#[macro_use]
mod macros;

mod ext;

mod message;
use message::{Message, Severity};

mod printer;
use printer::MessagePrinter;

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

    match args.command {
        Some(cmd) => {
            let command = &cmd[0];
            let cmd_args = &cmd[1..];

            let exit_status = run_command(printer.into(), &command, cmd_args)?;
            Ok(exit_status.code().unwrap_or(255))
        }
        None => {
            // read line by line from standard input
            printer_loop(io::stdin(), printer.as_ref(), Severity::Default);
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

fn printer_loop<R: Read>(reader: R, printer: &dyn MessagePrinter, default_severity: Severity) {
    let reader = BufReader::new(reader);
    for line in reader.lines().map(Result::unwrap) {
        // try to parse line as JSON, or create standard message from raw line
        let mut message: Message =
            serde_json::from_str(&line).unwrap_or_else(|_| Message::from_raw(line));

        message.set_default_severity(default_severity);

        printer.print(&message);
    }
}

fn run_command(
    printer: Arc<dyn MessagePrinter>,
    command: &str,
    cmd_args: &[String],
) -> Result<ExitStatus, Box<dyn Error>> {
    let mut child = Command::new(command)
        .args(cmd_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("take stdout");
    let stderr = child.stderr.take().expect("take stderr");

    // spawn threads
    let stdout_jh = {
        let printer = printer.clone();
        thread::spawn(move || printer_loop(stdout, printer.as_ref(), Severity::Default))
    };
    let stderr_jh = {
        let printer = printer.clone();
        thread::spawn(move || printer_loop(stderr, printer.as_ref(), Severity::Error))
    };

    // wait for process to stop
    let exit_status = child.wait()?;

    // join threads
    stdout_jh.join().unwrap();
    stderr_jh.join().unwrap();

    // success
    Ok(exit_status)
}
