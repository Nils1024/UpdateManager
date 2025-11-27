mod comm;
mod util;

use std::{env, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    // let options: Vec<String> = Vec::new();
    
    if args.len() == 1 {
        util::cli::print_help();
        return;
    } else {
        for option in args.iter() {
            let opt_lower = option.to_lowercase();

            if opt_lower == "help" {
                util::cli::print_help();
            } else if opt_lower == "startserver" {
                util::process_handling::start_new_process("target/debug/server", "server");
            } else if opt_lower == "stopserver" {
                util::process_handling::shutdown_process("server");
            } else if opt_lower == "start-client" {
                comm::client::connect();
            } else if opt_lower == "create-client-binary" {
                let _ = color_eyre::install();
                let terminal = ratatui::init();
                let _ = util::tui::tui::run(terminal);
                ratatui::restore();
            }
        }
    }

    return;
}
