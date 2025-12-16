use std::env;
use std::io::stdin;
use std::process::exit;
use update_manager::{comm, util};
use update_manager::util::config::get_config;
use update_manager::util::resource_bundle::resource_bundle;

fn main() {
    if !util::config::does_config_exists() {
        get_config();

        if !util::config::does_config_exists() {
            eprintln!("Failed to create config file");
            return;
        } else {
            println!("{}\n{}", 
                     resource_bundle::get_string(util::constants::RBC_CONFIG_CREATED),
                     resource_bundle::get_string(util::constants::RBC_CONTINUE));
            let mut input = String::new();
            stdin().read_line(&mut input).expect("Did not enter a correct string");

            if input.trim().to_lowercase() == "y" {
                exit(1);
            }
        }
    }

    let connection_result = comm::client::connect();

    if !connection_result {
        eprintln!("{}\n{}", 
                  resource_bundle::get_string(util::constants::RBC_CONNECTION_FAILED),
                  resource_bundle::get_string(util::constants::RBC_CONNECTION_FAILED_SOLUTION));
        exit(1);
    }

    if !get_config().contains_key("program") {
        println!("No program specified. Exiting");
        exit(1);
    } else {
        let mut args: Vec<String> = Vec::new();

        let mut counter = 0;
        loop {
            let arg_key = format!("arg{}", counter);

            if get_config().contains_key(&arg_key) {
                args.push(get_config()[&arg_key].to_string());
                counter += 1;
            } else {
                break;
            }
        }

        env::set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();
        util::process_handling::execute(get_config().get(util::constants::CONFIG_PROGRAM_KEY).unwrap(), args);
    }
}