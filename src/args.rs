use std::env;

use clap::clap_app;
use regex::Regex;

use crate::message::Severity;

#[derive(Clone, Debug)]
pub struct Args {
    pub debug: bool,
    pub compact: bool,

    pub severity_range: SeverityRange,

    pub raw: bool,

    pub show_context: bool,
    pub show_source: bool,

    pub grep: Option<String>,
    pub grep_show_all: bool,

    pub show_request_id: bool,
    pub show_process: bool,

    pub request_id: Option<String>,

    pub command: Option<Vec<String>>,

    pub plain: bool,
    pub colored: Option<bool>,
}

pub type SeverityRange = (Option<Severity>, Option<Severity>);

impl Args {
    fn app<'a, 'b>() -> clap::App<'a, 'b> {
        clap_app!(("pretty-log") =>
            (@arg debug: -d --debug
                "Enable debug output")
            (@arg compact: -z --compact
                "Enable compact formatting")

            (@arg min_level: -L --level [level] {Severity::validate_str})
            (@arg max_level: --("max-level") [level] {Severity::validate_str})

            (@arg show_request_id: -r --("show-request")
                "Show request ID")
            (@arg process: -P --process
                "Show process name")

            (@arg request_id: -R --request [id]
                "Filter by request ID")

            (@arg raw: -Z --raw
                "Do not parse JSON")

            (@arg context: -x --context
                "Show context data for logs")
            (@arg source: -s --source
                "Show source location for logs")

            (@arg grep: -g --grep [PATTERN] {is_regex}
                "Filter log entries matching expression")
            (@arg grep_show_all: -G --("grep-show-all")
                "Show non-matching lines as well when using --grep")

            (@arg plain: -p --plain
                "Only output text data")
            (@group color_mode =>
                (@arg color: -c --color
                    "Output pretty-printed in color")
                (@arg nocolor: -C --("no-color")
                    "Output pretty-printed uncolored")
            )

            (@arg command: +last +multiple
                "Run a command and pretty-print logs from it")
        )
    }

    /// Parse arguments from command line.
    ///
    /// If the `PRETTY_LOG_ARGS` environment variable is set, it will be split
    /// by whitespace and inserted before all arguments from the command line.
    pub fn parse() -> Args {
        let args = {
            let mut cmd_args: Vec<String> = env::args().collect();
            let mut args = Vec::new();

            // zeroeth argument: program name
            args.push(cmd_args.remove(0));

            // then, any arguments from the PRETTY_LOG_ARGS environment variable
            if let Ok(env_args) = env::var("PRETTY_LOG_ARGS") {
                args.extend(env_args.split_whitespace().map(str::to_owned));
            }

            // then, --request REQUEST_ID if the REQUEST_ID environment variable
            // is set
            if let Ok(request_id) = env::var("REQUEST_ID") {
                args.push("--request".to_string());
                args.push(request_id);
            }

            // then, any arguments from the command line
            args.extend(cmd_args.into_iter());

            args
        };

        // parse arguments with clap
        let matches = Self::app().get_matches_from(args);

        let min_level = matches
            .value_of("min_level")
            .map(Severity::try_parse_str)
            .transpose()
            .unwrap();
        let max_level = matches
            .value_of("max_level")
            .map(Severity::try_parse_str)
            .transpose()
            .unwrap();

        Args {
            debug: matches.is_present("debug"),
            compact: matches.is_present("compact"),

            severity_range: (min_level, max_level),

            raw: matches.is_present("raw"),

            show_context: matches.is_present("context"),
            show_source: matches.is_present("source"),

            show_request_id: matches.is_present("show_request_id"),
            show_process: matches.is_present("process"),

            request_id: matches.value_of("request_id").map(str::to_string),

            command: matches
                .values_of("command")
                .map(|cmd| cmd.map(str::to_string).collect()),

            grep: matches.value_of("grep").map(str::to_string),
            grep_show_all: matches.is_present("grep_show_all"),

            plain: matches.is_present("plain"),
            colored: {
                if matches.is_present("color") {
                    Some(true)
                } else if matches.is_present("nocolor") {
                    Some(false)
                } else {
                    None
                }
            },
        }
    }
}

fn is_regex(value: String) -> Result<(), String> {
    Regex::new(&value)
        .map(|_| ())
        .map_err(|err| err.to_string())
}
