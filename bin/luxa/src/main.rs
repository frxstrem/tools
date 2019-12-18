mod modes;

use clap::clap_app;
use luxa_core::{error, hid::LuxaforHid};

use std::process::exit;

fn main() {
    let args = clap_app!(luxa =>
        (@arg mode: [mode])
    )
    .get_matches();

    let all_modes = modes::get_all_modes();

    if let Some(mode_name) = args.value_of("mode") {
        let mode = all_modes
            .iter()
            .filter(move |m| m.names().iter().any(|name| *name == mode_name))
            .nth(0);

        match mode {
            Some(mode) => {
                let result =
                    LuxaforHid::open_default().and_then(move |device| mode.run(&device));

                if let Err(err) = result {
                    eprintln!("Error: {}", err);
                    exit(1);
                }
            }
            None => {
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
}
