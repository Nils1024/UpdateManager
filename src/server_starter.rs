use std::env;
use update_manager::comm::server::{Server};
use update_manager::util;
use update_manager::util::config::get_config;

fn new_server_with_process() {
    util::process_handling::start_new_process(env::current_exe().unwrap(),
                                              vec![util::constants::ARG_START_SERVER],
                                              util::constants::SERVER_PROCESS_DESCRIPTION);
}

fn stop_server_process() {
    util::process_handling::shutdown_process(util::constants::SERVER_PROCESS_DESCRIPTION);
}

fn start_server() {
    let mut server = Server::new(
        format!("{}:{}",
                util::constants::STD_SERVER_ADDRESS,
                get_config().get(util::constants::CONFIG_PORT_KEY).unwrap()));

    println!("Server started at {}", get_config().get(util::constants::CONFIG_PORT_KEY).unwrap());
    println!("Hash: {}", server.get_hash());

    server.start().expect("Server stopped");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // let options: Vec<String> = Vec::new();

    if args.len() == 2 {
        for option in args.iter() {
            let opt_lower = option.to_lowercase();

            if opt_lower == util::constants::ARG_HELP {
                util::cli::print_help();
            } else if opt_lower == util::constants::ARG_START_SERVER_PROCESS {
                new_server_with_process();
            } else if opt_lower == util::constants::ARG_STOP_SERVER_PROCESS {
                start_server()
            } else if opt_lower == util::constants::ARG_START_SERVER {
                start_server()
            }
        }
    } else if args.len() == 1 {
        if util::process_handling::is_process_running(util::constants::SERVER_PROCESS_DESCRIPTION) {
            stop_server_process();
        } else {
            new_server_with_process();
        }
    }
}