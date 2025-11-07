mod comm;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let options: Vec<String> = Vec::new();
    
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