mod comm;
mod util;

use std::{env, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    // let options: Vec<String> = Vec::new();
    
    if args.len() == 1 {
        print_help();
        return;
    } else {
        for option in args.iter() {
            let opt_lower = option.to_lowercase();

            if opt_lower == "test" {
                println!("test");
                
            } else if opt_lower == "startserver" {
                comm::server::start().unwrap();
            } else if opt_lower == "get-dir-hash" {
                println!("{}", util::hash::get_dir_hash(Path::new("./")));
            } else if opt_lower == "create-config" {
                if util::config::does_config_exists() {
                    util::config::read_config(Path::new("./"));
                } else {
                    util::config::write_default_config(Path::new("./"));
                }
            } else if opt_lower == "start-client" {
                comm::client::connect();
            }
        }
    }

    return;
}

/// Prints out the help string to the console
fn print_help() {
    println!("Update Manager");
    println!("usage: upman [options]");
    println!("options:");
    println!("\t-v, --verbose\tAdds logging");
    return;
}