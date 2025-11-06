mod util;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let options: Vec<String> = Vec::new();
    
    if args.len() == 1 {
        print_help();
        return;
    } else {
        for option in args.iter() {
            if option == "Test" {
                println!("test");
                
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