use std::env;
use std::io::stdin;
use std::process::exit;
use update_manager::{comm, util};
use update_manager::util::config::get_config;

fn main() {
    if !util::config::does_config_exists() {
        get_config();

        if !util::config::does_config_exists() {
            eprintln!("Failed to create config file");
            return;
        } else {
            println!("Config file created. Would you like to continue? [y/n]");
            let mut input = String::new();
            stdin().read_line(&mut input).expect("Did not enter a correct string");

            if input.trim().to_lowercase() == "y" {
                exit(1);
            }
        }
    }

    let connection_result = comm::client::connect();

    if !connection_result {
        eprintln!("Failed to connect to server. Make sure the server is running or you configured the correct address in the upman.json");
        exit(1);
    }

    if !get_config().contains_key("program") {
        println!("No program specified. Exiting");
        exit(1);
    } else {
        let mut args: Vec<String> = Vec::new();

        let mut counter = 0;
        loop {
            if get_config().contains_key(format!("arg{}", counter).as_str()) {
                args.push(get_config()[format!("arg{}", counter).as_str()].to_string());
                counter += 1;
            } else {
                break;
            }
        }

        env::set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();
        util::process_handling::execute(get_config().get("program").unwrap(), args);
    }
}