mod modes;

use clap::clap_app;
use luxa_core::{error::LuxaError, hid::LuxaforHid};

use std::process::exit;

#[tokio::main]
async fn main() {
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
                let result = async move {
                    let device = LuxaforHid::open_default()?;
                    mode.run(&device).await?;
                    Ok::<_, LuxaError>(())
                }.await;

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
