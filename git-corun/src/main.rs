use std::error::Error;
use std::fmt::Display;
use std::io::{self, Write};
use std::path::Path;
use std::process::{self, Command, ExitStatus, Stdio};

use clap::clap_app;
use tempdir::TempDir;

mod git;

fn main() {
    let args = Args::parse().unwrap_or_else(|err| {
        if err.use_stderr() {
            eprintln!("{}", err.message);
            process::exit(128);
        }
        let out = io::stdout();
        writeln!(&mut out.lock(), "{}", err.message).expect("Error writing Error to stdout");
        process::exit(128);
    });
    match app(args) {
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(128);
        }
        Ok(exit_code) => process::exit(exit_code),
    }
}

fn app(args: Args) -> Result<i32, Box<dyn Error>> {
    // get git directory
    let git_dir = git::get_git_dir()?;

    // get latest stash commit
    let stash_commit = if args.apply_stash {
        Some(git::get_commit_hash(&git_dir, "refs/stash")?)
    } else if args.apply_index {
        unimplemented!()
    } else {
        None
    };

    // expand list of commits
    let commits = args
        .commits
        .iter()
        .map(|commit| git::get_commit_hashes(&git_dir, commit))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // create temporary directory
    let tmpdir = TempDir::new("git-corun")?;

    // git clone into temporary directory
    git::clone_local(&git_dir, &tmpdir)?;

    let mut last_exit_code = 0;
    for commit in commits {
        last_exit_code = run_app_for(
            &args,
            &git_dir,
            tmpdir.as_ref(),
            &commit,
            stash_commit.as_ref().map(String::as_str),
        )?;
    }

    Ok(last_exit_code)
}

fn run_app_for(
    args: &Args,
    git_dir: &Path,
    work_tree: &Path,
    commit: &str,
    stash_commit: Option<&str>,
) -> Result<i32, Box<dyn Error>> {
    // get commit hash
    let commit = git::get_commit_hash(&git_dir, commit)?;

    // check out directory
    git::checkout_detached(&work_tree, &commit)?;

    // clean directory
    git::clean_work_dir(&work_tree)?;

    if let Some(stash_commit) = stash_commit {
        // apply stash
        git::apply_stash(&work_tree, stash_commit)?;
    }

    // print commit
    print_commit(&git_dir, &commit, Some(Status::Pending))?;

    // run command in repo
    let exit_status = run_in(&args, args.command.iter().map(String::as_str), &work_tree)?;
    let status = if exit_status.success() {
        Status::Success
    } else {
        Status::Failure
    };

    // print status
    if !args.verbose {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        write!(stdout, "\x1b[1F\x1b[K")?;
        stdout.flush()?;
    }
    print_commit(&git_dir, &commit, Some(status))?;

    Ok(exit_status.code().unwrap_or(255))
}

fn print_commit(
    git_dir: impl AsRef<Path>,
    commit: impl AsRef<str>,
    status: Option<Status>,
) -> io::Result<()> {
    let base_format = "%C(yellow)%h %C(bold)%G? %Creset%C(cyan)[%Cgreen%ad%C(cyan) by %Cred%an%C(cyan)]%Creset %s";
    let format = if let Some(status) = status {
        format!("{} {}", status.get_format(), base_format)
    } else {
        format!("{}", base_format)
    };

    git::show_commit(git_dir, commit, &format)
}

fn run_in<'a, I>(args: &Args, command: I, dir: impl AsRef<Path>) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut command = command.into_iter();
    let dir = dir.as_ref();

    let cmd_first = command.next().unwrap();
    let cmd_rest = command.collect::<Vec<_>>();

    let (exec_name, cmd_args) = if args.shell_command {
        let exec_name = "/bin/bash";
        let mut args = vec!["-c", cmd_first, "--"];
        args.extend(cmd_rest);
        (exec_name, args)
    } else {
        (cmd_first, cmd_rest)
    };

    if args.verbose {
        Command::new(&exec_name)
            .args(&cmd_args)
            .current_dir(&dir)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    } else {
        Command::new(&exec_name)
            .args(&cmd_args)
            .current_dir(&dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }
}

#[derive(Copy, Clone, Debug)]
enum Status {
    Pending,
    Success,
    Failure,
}

impl Status {
    fn get_format(self) -> impl Display {
        match self {
            Status::Pending => "%C(bold)%C(yellow)●%Creset",
            Status::Success => "%C(bold)%C(green)✔%Creset",
            Status::Failure => "%C(bold)%C(red)✘%Creset",
        }
    }
}

struct Args {
    apply_stash: bool,
    apply_index: bool,
    shell_command: bool,
    verbose: bool,

    commits: Vec<String>,
    command: Vec<String>,
}

impl Args {
    fn parse() -> clap::Result<Args> {
        let matches = clap_app!(("git-corun") =>
            (@group apply =>
                (@arg apply_stash: -s --stash      "Apply latest stash before running")
                // (@arg apply_index: -i --index      "Apply index before running")
            )
            (@arg shell_command: -c            "Run as shell command")
            (@arg verbose: -v --verbose        "Show output from commands")
            (@arg commits: [commits] ...       "List of commits to run on")
            (@arg command: <command> ... +last "Command to execute")
        )
        .get_matches_safe()?;

        Ok(Args {
            apply_stash: matches.is_present("apply_stash"),
            apply_index: false,
            shell_command: matches.is_present("shell_command"),
            verbose: matches.is_present("verbose"),

            commits: matches
                .values_of("commits")
                .map(|commits| commits.map(str::to_string).collect())
                .unwrap_or_else(|| vec!["HEAD".to_string()]),
            command: matches
                .values_of("command")
                .unwrap()
                .map(str::to_string)
                .collect(),
        })
    }
}
