mod modes;
mod error;
mod luxa;

use clap::clap_app;
use std::process::exit;

use crate::error::*;
use crate::luxa::*;

#[tokio::main]
async fn main() {
    let device = match LuxaforHid::open_default() {
        Ok(device) => device,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    };

    let args = clap_app!(luxa =>
        (@arg mode: [mode])
    )
    .get_matches();

    if let Err(err) = run(&device, &args).await {
        eprintln!("Error: {}", err);
        exit(1);
    }
}

async fn run(device: &LuxaforHid, args: &clap::ArgMatches<'_>) -> Result<(), LuxaError> {
    let all_modes = modes::get_all_modes();

    if let Some(mode_name) = args.value_of("mode") {
        let mode = all_modes
            .iter()
            .filter(move |m| m.names().iter().any(|name| *name == mode_name))
            .nth(0);

        match mode {
            Some(mode) => {
                mode.run(device).await?;
            }
            None => {
                // TODO: use error
                eprintln!("Error: Unknown mode {:?}", mode_name);
                exit(2);
            }
        }
    } else {
        println!("Modes:");
        for mode in all_modes {
            let names = mode.names().join(", ");
            println!(" - {}", names);
        }
    }

    Ok(())
}
