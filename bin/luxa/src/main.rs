mod error;
mod luxa;
mod modes;

use std::env;
use std::process::exit;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    let all_modes = modes::get_all_modes();

    match args.len() {
        0 => {
            println!("Modes:");
            for mode in all_modes {
                let names = mode.names().join(", ");
                println!(" - {}", names);
            }
        }

        1 => {
            let mode_name = &args[0];

            let mode = all_modes
                .iter()
                .filter(move |m| m.names().iter().any(|name| *name == mode_name))
                .nth(0);

            match mode {
                Some(mode) => {
                    let result =
                        luxa::Luxafor::open_default().and_then(move |device| mode.run(&device));

                    if let Err(err) = result {
                        eprintln!("Error: {}", err);
                        exit(1);
                    }
                }
                None => {
                    eprintln!("Error: Unknown mode {:?}", &args[0]);
                    exit(2);
                }
            }
        }

        _ => {
            eprintln!("Error: Too many arguments");
            exit(2);
        }
    }
}
