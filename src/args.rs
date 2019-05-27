use std::env;

use clap::clap_app;

pub struct Args {}

impl Args {
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
        let matches = clap_app!(("pretty-log") =>
        ).get_matches_from(args);

        Args {}
    }
}
