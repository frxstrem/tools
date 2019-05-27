use std::env;

use clap::clap_app;

#[derive(Clone, Debug)]
pub struct Args {
    pub debug: bool,

    pub show_context: bool,
    pub show_source: bool,

    pub show_request_id: bool,
    pub show_process: bool,

    pub formatting: FormattingMode,
}

impl Args {
    fn app<'a, 'b>() -> clap::App<'a, 'b> {
        clap_app!(("pretty-log") =>
            (@arg debug: -d --debug             "Enable debug output")

            (@arg request: -r --request         "Show request ID")
            (@arg process: -P --process         "Show process name")

            (@arg context: -x --context         "Show context data for logs")
            (@arg source: -s --source           "Show source location for logs")

            (@arg plain: -p --plain             "Only output text data")
            (@arg color: -c --color             "Enable color")
            (@arg nocolor: -C --("no-color")    "Disable color")
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

            // then, any arguments from the command line
            args.extend(cmd_args.into_iter());

            args
        };

        // parse arguments with clap
        let matches = Self::app().get_matches_from(args);

        Args {
            debug: matches.is_present("debug"),

            show_context: matches.is_present("context"),
            show_source: matches.is_present("source"),

            show_request_id: matches.is_present("request"),
            show_process: matches.is_present("process"),

            formatting: {
                if matches.is_present("plain") {
                    FormattingMode::Plain
                } else if matches.is_present("color") {
                    FormattingMode::Colored
                } else if matches.is_present("nocolor") {
                    FormattingMode::Uncolored
                } else {
                    FormattingMode::Auto
                }
            },
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FormattingMode {
    Auto,
    Colored,
    Uncolored,
    Plain,
}
