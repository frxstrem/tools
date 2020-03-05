mod ext;
mod format;
mod message;
mod parse;
mod utils;

use crossbeam::scope;
use std::ffi::OsStr;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Mutex;
use structopt::StructOpt;

use crate::format::{style::*, InputFormat, OutputFormat};
use crate::message::{Message, Severity};
use crate::utils::is_stdout_tty;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short = "i", long = "input", use_delimiter = true, possible_values = format::get_input_format_variants(), default_value = format::get_input_format_default())]
    input_format: Vec<String>,

    #[structopt(short = "o", long = "output", possible_values = format::get_output_format_variants(), default_value = format::get_output_format_default())]
    output_format: String,

    #[structopt(flatten)]
    display_options: DisplayOptions,

    #[structopt(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(Clone, Debug, StructOpt)]
pub struct DisplayOptions {
    #[structopt(short = "x", long = "context")]
    show_context: bool,

    #[structopt(short = "c", long = "compact")]
    compact: bool,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    #[structopt(external_subcommand)]
    External(Vec<String>),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::from_args();

    let writer = io::stdout();
    let style: AnyStyle = if is_stdout_tty() {
        ColoredStyle.into()
    } else {
        PlainStyle.into()
    };

    let input_format = format::get_input_format(&opts.input_format)?;

    let output_format =
        format::get_output_format(&opts.output_format, style, &opts.display_options)?;

    match opts.subcommand.as_ref() {
        Some(Subcommand::External(args)) => {
            run_command(writer, input_format, output_format, args)?;
        }
        None => {
            run(
                io::stdin(),
                &Mutex::new(writer),
                input_format,
                output_format,
                Severity::Default,
            )?;
        }
    }

    Ok(())
}

fn run(
    reader: impl Read,
    writer: &Mutex<impl Write>,
    input: impl InputFormat,
    output: impl OutputFormat,
    default_severity: Severity,
) -> io::Result<()> {
    let mut reader = BufReader::new(reader);

    let mut line = String::new();
    while {
        line.clear();
        reader.read_line(&mut line)? > 0
    } {
        let line = line.trim_end_matches('\n');

        let message = input
            .parse_message(line, default_severity)
            .unwrap_or_else(|| Message::from_text(line, default_severity))
            .trim();

        let writer = &mut *writer.lock().unwrap();
        output.print_message(writer, &message)?;
    }

    Ok(())
}

fn run_command(
    writer: impl Write + Send + Sync,
    input: impl InputFormat,
    output: impl OutputFormat,
    command: &[impl AsRef<OsStr>],
) -> io::Result<ExitStatus> {
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("take stdout");
    let stderr = child.stderr.take().expect("take stderr");

    let writer = Mutex::new(writer);

    scope(|s| {
        s.spawn(|_| run(stdout, &writer, &input, &output, Severity::Info));
        s.spawn(|_| run(stderr, &writer, &input, &output, Severity::Error));
        child.wait()
    })
    .unwrap()
    .map_err(|err| err.into())
}
